use anyhow::{Context, Result};
use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
        let total = self.total_reads();
        if total > 0 {
            Some(self.methylated_reads as f32 / total as f32)
        } else {
            None
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

    /// Efficiently read Bismark file - ported from R::read_bdg
    pub fn read(&self) -> Result<Vec<BismarkRecord>> {
        if self.file_path.ends_with(".gz") {
            self.read_gzipped()
        } else {
            self.read_mmap()
        }
    }

    fn read_gzipped(&self) -> Result<Vec<BismarkRecord>> {
        let file = File::open(&self.file_path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let reader = BufReader::new(decoder);

        self.parse_reader(reader)
    }

    fn read_mmap(&self) -> Result<Vec<BismarkRecord>> {
        let file = File::open(&self.file_path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        let mut records = Vec::new();
        for line in mmap.split(|&b| b == b'\n') {
            if let Ok(line_str) = std::str::from_utf8(line) {
                if let Some(record) = self.parse_line(line_str) {
                    records.push(record);
                }
            }
        }

        Ok(records)
    }

    fn parse_reader<R: std::io::Read>(&self, reader: BufReader<R>) -> Result<Vec<BismarkRecord>> {
        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if let Some(record) = self.parse_line(&line) {
                records.push(record);
            }
        }
        Ok(records)
    }

    /// Parse Bismark line format
    /// Format: chr start end meth_reads unmeth_reads context
    fn parse_line(&self, line: &str) -> Option<BismarkRecord> {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 6 {
            return None;
        }

        // Bismark uses 1-based coordinates
        let chr = parts[0].to_string();
        let start: u32 = parts[1].parse().ok()?;
        let meth_reads: u32 = parts[3].parse().ok()?;
        let unmeth_reads: u32 = parts[4].parse().ok()?;

        Some(BismarkRecord {
            chr,
            start: start - 1, // Convert to 0-based
            methylated_reads: meth_reads,
            unmethylated_reads: unmeth_reads,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bismark_line() {
        let reader = BismarkReader::new("test.txt".to_string());
        let line = "chr1\t10469\t10470\t0\t10\tCG";
        let record = reader.parse_line(line);

        assert!(record.is_some());
        let rec = record.unwrap();
        assert_eq!(rec.chr, "chr1");
        assert_eq!(rec.start, 10468); // 10469 - 1 (0-based)
        assert_eq!(rec.methylated_reads, 0);
        assert_eq!(rec.unmethylated_reads, 10);
        assert_eq!(rec.total_reads(), 10);
        assert_eq!(rec.beta_value(), Some(0.0));
    }

    #[test]
    fn test_beta_value_calculation() {
        let record = BismarkRecord {
            chr: "chr1".to_string(),
            start: 100,
            methylated_reads: 5,
            unmethylated_reads: 5,
        };
        assert_eq!(record.beta_value(), Some(0.5));
    }
}
