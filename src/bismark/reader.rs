use anyhow::{anyhow, bail, Context, Result};
use memmap2::Mmap;
use std::fs::File;
use std::io::{BufRead, BufReader};

const PERCENTAGE_TOLERANCE: f64 = 0.01;

#[derive(Debug, Clone)]
pub struct BismarkRecord {
    pub chr: String,
    pub start: u32,
    pub methylated_reads: u32,
    pub unmethylated_reads: u32,
}

impl BismarkRecord {
    pub fn total_reads(&self) -> u32 {
        self.methylated_reads + self.unmethylated_reads
    }

    pub fn beta_value(&self) -> Option<f32> {
        let total_reads = self.total_reads();
        if total_reads == 0 {
            None
        } else {
            Some(self.methylated_reads as f32 / total_reads as f32)
        }
    }
}

pub struct BismarkReader {
    file_path: String,
}

impl BismarkReader {
    pub fn new(file_path: String) -> Self {
        Self { file_path }
    }

    /// Read a standard six-column Bismark coverage file.
    pub fn read(&self) -> Result<Vec<BismarkRecord>> {
        if self.file_path.ends_with(".gz") {
            self.read_gzipped()
        } else {
            self.read_mmap()
        }
    }

    fn read_gzipped(&self) -> Result<Vec<BismarkRecord>> {
        let file = File::open(&self.file_path)
            .with_context(|| format!("Failed to open Bismark file {}", self.file_path))?;
        let decoder = flate2::read::GzDecoder::new(file);
        self.parse_reader(BufReader::new(decoder))
    }

    fn read_mmap(&self) -> Result<Vec<BismarkRecord>> {
        let file = File::open(&self.file_path)
            .with_context(|| format!("Failed to open Bismark file {}", self.file_path))?;
        let mmap = unsafe { Mmap::map(&file)? };

        let mut records = Vec::new();
        for (line_index, line) in mmap.split(|&byte| byte == b'\n').enumerate() {
            if line.is_empty() {
                continue;
            }
            let line_number = line_index + 1;
            let line_text = std::str::from_utf8(line).with_context(|| {
                format!(
                    "Invalid UTF-8 in {} at line {}",
                    self.file_path, line_number
                )
            })?;
            records.push(self.parse_line(line_text, line_number)?);
        }

        Ok(records)
    }

    fn parse_reader<R: std::io::Read>(&self, reader: BufReader<R>) -> Result<Vec<BismarkRecord>> {
        let mut records = Vec::new();
        for (line_index, line_result) in reader.lines().enumerate() {
            let line_number = line_index + 1;
            let line = line_result.with_context(|| {
                format!("Failed to read {} at line {}", self.file_path, line_number)
            })?;
            if line.is_empty() {
                continue;
            }
            records.push(self.parse_line(&line, line_number)?);
        }
        Ok(records)
    }

    /// Parse the standard Bismark coverage format:
    /// chromosome, start, end, methylation percentage, methylated count,
    /// unmethylated count. Coordinates are 1-based in the input.
    fn parse_line(&self, line: &str, line_number: usize) -> Result<BismarkRecord> {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 6 {
            bail!(
                "Invalid Bismark coverage record in {} at line {}: expected exactly 6 tab-separated columns, got {}",
                self.file_path,
                line_number,
                fields.len()
            );
        }

        let chromosome = fields[0].trim();
        if chromosome.is_empty() {
            bail!(
                "Invalid Bismark coverage record in {} at line {}: chromosome is empty",
                self.file_path,
                line_number
            );
        }

        let start_1based = parse_field::<u32>(fields[1], "start", &self.file_path, line_number)?;
        let end_1based = parse_field::<u32>(fields[2], "end", &self.file_path, line_number)?;
        let methylation_percentage = parse_field::<f64>(
            fields[3],
            "methylation percentage",
            &self.file_path,
            line_number,
        )?;
        let methylated_reads =
            parse_field::<u32>(fields[4], "methylated count", &self.file_path, line_number)?;
        let unmethylated_reads = parse_field::<u32>(
            fields[5],
            "unmethylated count",
            &self.file_path,
            line_number,
        )?;

        if start_1based == 0 {
            bail!(
                "Invalid Bismark coverage record in {} at line {}: start must be >= 1",
                self.file_path,
                line_number
            );
        }
        if end_1based < start_1based {
            bail!(
                "Invalid Bismark coverage record in {} at line {}: end {} is before start {}",
                self.file_path,
                line_number,
                end_1based,
                start_1based
            );
        }
        if !methylation_percentage.is_finite() || !(0.0..=100.0).contains(&methylation_percentage) {
            bail!(
                "Invalid Bismark coverage record in {} at line {}: methylation percentage must be finite and between 0 and 100",
                self.file_path,
                line_number
            );
        }

        let total_reads = methylated_reads
            .checked_add(unmethylated_reads)
            .ok_or_else(|| {
                anyhow!(
                    "Coverage overflow in {} at line {}: {} + {} exceeds u32",
                    self.file_path,
                    line_number,
                    methylated_reads,
                    unmethylated_reads
                )
            })?;
        let expected_percentage = if total_reads == 0 {
            0.0
        } else {
            100.0 * methylated_reads as f64 / total_reads as f64
        };
        if (expected_percentage - methylation_percentage).abs() > PERCENTAGE_TOLERANCE {
            bail!(
                "Inconsistent Bismark coverage record in {} at line {}: reported percentage {:.6} does not match counts {}/{} (expected {:.6})",
                self.file_path,
                line_number,
                methylation_percentage,
                methylated_reads,
                unmethylated_reads,
                expected_percentage
            );
        }

        Ok(BismarkRecord {
            chr: chromosome.to_string(),
            start: start_1based - 1,
            methylated_reads,
            unmethylated_reads,
        })
    }
}

fn parse_field<T>(raw_value: &str, field_name: &str, path: &str, line_number: usize) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    raw_value.trim().parse::<T>().map_err(|error| {
        anyhow!(
            "Invalid {} in {} at line {}: {:?} ({})",
            field_name,
            path,
            line_number,
            raw_value,
            error
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_bismark_coverage_line() {
        let reader = BismarkReader::new("test.cov".to_string());
        let record = reader
            .parse_line("chr1\t10469\t10469\t20.000000\t2\t8", 1)
            .unwrap();

        assert_eq!(record.chr, "chr1");
        assert_eq!(record.start, 10468);
        assert_eq!(record.methylated_reads, 2);
        assert_eq!(record.unmethylated_reads, 8);
        assert_eq!(record.total_reads(), 10);
        assert_eq!(record.beta_value(), Some(0.2));
    }

    #[test]
    fn preserves_coverage_above_u16_range() {
        let reader = BismarkReader::new("test.cov".to_string());
        let record = reader
            .parse_line("chr1\t10\t10\t50.000000\t35000\t35000", 1)
            .unwrap();

        assert_eq!(record.total_reads(), 70_000);
        assert_eq!(record.beta_value(), Some(0.5));
    }

    #[test]
    fn rejects_inconsistent_percentage_and_counts() {
        let reader = BismarkReader::new("test.cov".to_string());
        let error = reader
            .parse_line("chr1\t10\t10\t90.0\t2\t8", 7)
            .unwrap_err();

        assert!(error.to_string().contains("line 7"));
        assert!(error.to_string().contains("does not match counts"));
    }

    #[test]
    fn rejects_nonstandard_column_count() {
        let reader = BismarkReader::new("test.cov".to_string());
        let error = reader.parse_line("chr1\t10\t10\t2\t8", 2).unwrap_err();

        assert!(error.to_string().contains("expected exactly 6"));
    }
}
