use anyhow::{Context, Result};
use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
};

use crate::atomic_output::write_atomically;

trait SeparatedString {
    fn separated_string(&self) -> String;
}

impl SeparatedString for usize {
    fn separated_string(&self) -> String {
        let s = self.to_string();
        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i).is_multiple_of(3) {
                result.push(',');
            }
            result.push(*c);
        }
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpGSite {
    pub chr: String,
    pub start: u32,
    pub end: u32,
    pub strand: char,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContigInfo {
    pub contig: String,
    pub length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpGData {
    pub cpgs: Vec<CpGSite>,
    pub contig_lens: Vec<ContigInfo>,
    pub release_name: String,
}

#[derive(Debug)]
struct CurrentContigExtraction {
    name: String,
    is_included: bool,
    sequence_length: usize,
    previous_base: Option<u8>,
}

pub struct CpGExtractor {
    fasta_path: String,
    contigs: Option<Vec<String>>,
}

impl CpGExtractor {
    pub fn new(fasta_path: String) -> Self {
        Self {
            fasta_path,
            contigs: None,
        }
    }

    pub fn contigs(mut self, contigs: Vec<String>) -> Self {
        self.contigs = Some(contigs);
        self
    }

    /// Extract CpG sites - ported from R::extract_CPGs
    pub fn extract(&self) -> Result<CpGData> {
        let reader = self.open_fasta_reader()?;
        self.extract_from_reader(reader)
    }

    fn open_fasta_reader(&self) -> Result<BufReader<Box<dyn Read>>> {
        let fasta_file = File::open(&self.fasta_path).context("Failed to open FASTA file")?;
        let reader: Box<dyn Read> = if self.fasta_path.ends_with(".gz") {
            Box::new(MultiGzDecoder::new(fasta_file))
        } else {
            Box::new(fasta_file)
        };

        Ok(BufReader::new(reader))
    }

    fn extract_from_reader<R: BufRead>(&self, mut reader: R) -> Result<CpGData> {
        let mut all_cpgs = Vec::new();
        let mut contig_info = Vec::new();
        let mut current_contig = None;
        let mut header_line = Vec::new();

        loop {
            header_line.clear();
            let header_bytes_read = reader
                .read_until(b'\n', &mut header_line)
                .context("Failed to read FASTA header")?;
            if header_bytes_read == 0 {
                break;
            }

            let header = header_line
                .strip_suffix(b"\n")
                .unwrap_or(&header_line)
                .strip_suffix(b"\r")
                .unwrap_or_else(|| header_line.strip_suffix(b"\n").unwrap_or(&header_line));
            if !header.starts_with(b">") {
                anyhow::bail!(
                    "Expected FASTA header beginning with '>', found sequence data before a header"
                );
            }

            self.finalize_contig(&mut current_contig, &mut contig_info)?;
            let contig_name = Self::parse_contig_name(header)?;
            current_contig = Some(CurrentContigExtraction {
                is_included: self.should_include_chr(&contig_name),
                name: contig_name,
                sequence_length: 0,
                previous_base: None,
            });

            loop {
                let buffer = reader.fill_buf().context("Failed to read FASTA sequence")?;
                if buffer.is_empty() || buffer[0] == b'>' {
                    break;
                }

                let bytes_to_consume = Self::consume_sequence_chunk(
                    buffer,
                    current_contig
                        .as_mut()
                        .expect("current contig is initialized"),
                    &mut all_cpgs,
                )?;
                reader.consume(bytes_to_consume);
            }
        }

        self.finalize_contig(&mut current_contig, &mut contig_info)?;
        println!(
            "-Done. Extracted {} CpGs from {} contigs.",
            all_cpgs.len().separated_string(),
            contig_info.len()
        );

        Ok(CpGData {
            cpgs: all_cpgs,
            contig_lens: contig_info,
            release_name: self.extract_genome_name()?,
        })
    }

    fn parse_contig_name(header: &[u8]) -> Result<String> {
        let contig_name = header[1..]
            .split(|byte| byte.is_ascii_whitespace())
            .next()
            .filter(|identifier| !identifier.is_empty())
            .context("FASTA header does not contain a contig identifier")?;

        std::str::from_utf8(contig_name)
            .context("FASTA contig identifier is not valid UTF-8")
            .map(str::to_owned)
    }

    fn consume_sequence_chunk(
        buffer: &[u8],
        current_contig: &mut CurrentContigExtraction,
        all_cpgs: &mut Vec<CpGSite>,
    ) -> Result<usize> {
        let bytes_to_consume = buffer
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(buffer.len(), |newline_index| newline_index + 1);

        if !current_contig.is_included {
            return Ok(bytes_to_consume);
        }

        for base in &buffer[..bytes_to_consume] {
            if base.is_ascii_whitespace() {
                continue;
            }

            let position = u32::try_from(current_contig.sequence_length).with_context(|| {
                format!(
                    "FASTA contig {} exceeds the u32 coordinate limit",
                    current_contig.name
                )
            })?;
            let normalized_base = base.to_ascii_uppercase();
            if current_contig.previous_base == Some(b'C') && normalized_base == b'G' {
                all_cpgs.push(CpGSite {
                    chr: current_contig.name.clone(),
                    start: position - 1,
                    end: position + 1,
                    strand: '+',
                });
            }
            current_contig.previous_base = Some(normalized_base);
            current_contig.sequence_length += 1;
        }

        Ok(bytes_to_consume)
    }

    fn finalize_contig(
        &self,
        current_contig: &mut Option<CurrentContigExtraction>,
        contig_info: &mut Vec<ContigInfo>,
    ) -> Result<()> {
        let Some(contig) = current_contig.take() else {
            return Ok(());
        };
        if !contig.is_included {
            return Ok(());
        }

        contig_info.push(ContigInfo {
            contig: contig.name,
            length: u32::try_from(contig.sequence_length)
                .context("FASTA contig exceeds the u32 coordinate limit")?,
        });
        Ok(())
    }

    fn should_include_chr(&self, chr: &str) -> bool {
        if let Some(ref contigs) = self.contigs {
            contigs.contains(&chr.to_string())
        } else {
            // Default to standard chromosomes (equivalent to standardChromosomes)
            self.is_standard_chromosome(chr)
        }
    }

    fn is_standard_chromosome(&self, chr: &str) -> bool {
        let chr_clean = chr
            .get(..3)
            .filter(|prefix| prefix.eq_ignore_ascii_case("chr"))
            .map_or(chr, |_| &chr[3..])
            .to_ascii_uppercase();

        matches!(
            chr_clean.as_str(),
            "1" | "2"
                | "3"
                | "4"
                | "5"
                | "6"
                | "7"
                | "8"
                | "9"
                | "10"
                | "11"
                | "12"
                | "13"
                | "14"
                | "15"
                | "16"
                | "17"
                | "18"
                | "19"
                | "20"
                | "21"
                | "22"
                | "X"
                | "Y"
                | "M"
                | "MT"
        )
    }

    fn extract_genome_name(&self) -> Result<String> {
        // Infer genome version from FASTA path.
        // e.g.: hg19.fa -> hg19, hg19.fa.gz -> hg19
        let path = std::path::Path::new(&self.fasta_path);
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let mut name = file_name.to_string();
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".gz") {
            name.truncate(name.len() - 3);
        }
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".fasta") {
            name.truncate(name.len() - 6);
        } else if lower.ends_with(".fna") || lower.ends_with(".fa") {
            name.truncate(name.len() - 3);
        }

        if name.is_empty() {
            Ok("unknown".to_string())
        } else {
            Ok(name)
        }
    }

    /// Save CpG data as RON format (for later use)
    pub fn save(&self, output_path: &str) -> Result<()> {
        let cpg_data = self.extract()?;

        let ron_string = ron::ser::to_string_pretty(&cpg_data, Default::default())
            .context("Failed to serialize CpG data")?;

        write_atomically(Path::new(output_path), |staging_path| {
            let mut writer = BufWriter::new(File::create(staging_path)?);
            writer.write_all(ron_string.as_bytes())?;
            writer.flush()?;
            Ok(())
        })?;

        println!("CpG data saved to: {}", output_path);
        Ok(())
    }
}

/// Load pre-extracted CpG data
pub fn load_cpg_data(path: &str) -> Result<CpGData> {
    let content = std::fs::read_to_string(path).context("Failed to read CpG data file")?;

    let cpg_data: CpGData = ron::from_str(&content).context("Failed to deserialize CpG data")?;

    Ok(cpg_data)
}

/// Extract and save CpG data (convenience function)
pub fn extract_and_save(
    genome: String,
    output: String,
    contigs: Option<Vec<String>>,
) -> Result<()> {
    let mut extractor = CpGExtractor::new(genome);
    if let Some(contigs) = contigs {
        extractor = extractor.contigs(contigs);
    }
    extractor.save(&output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_chromosome_detection() {
        let extractor = CpGExtractor::new("test.fa".to_string());

        assert!(extractor.is_standard_chromosome("chr1"));
        assert!(extractor.is_standard_chromosome("Chr1"));
        assert!(extractor.is_standard_chromosome("chrX"));
        assert!(extractor.is_standard_chromosome("chrY"));
        assert!(extractor.is_standard_chromosome("chrM"));
        assert!(extractor.is_standard_chromosome("MT"));
        assert!(!extractor.is_standard_chromosome("chrUn"));
    }

    #[test]
    fn test_extract_cpgs_from_unwrapped_fasta_records() {
        let extractor = CpGExtractor::new("test.fa".to_string());
        let fasta = b">chr1 source\nACCG\n>chrUn\nCG\n>chr2\nCGCG";

        let cpg_data = extractor
            .extract_from_reader(BufReader::with_capacity(3, &fasta[..]))
            .expect("extract CpGs from unwrapped FASTA");

        assert_eq!(cpg_data.contig_lens.len(), 2);
        assert_eq!(cpg_data.contig_lens[0].contig, "chr1");
        assert_eq!(cpg_data.contig_lens[0].length, 4);
        assert_eq!(cpg_data.contig_lens[1].contig, "chr2");
        assert_eq!(cpg_data.contig_lens[1].length, 4);
        assert_eq!(cpg_data.cpgs.len(), 3);
        assert_eq!(cpg_data.cpgs[0].chr, "chr1");
        assert_eq!(cpg_data.cpgs[0].start, 2);
        assert_eq!(cpg_data.cpgs[1].chr, "chr2");
        assert_eq!(cpg_data.cpgs[1].start, 0);
        assert_eq!(cpg_data.cpgs[2].start, 2);
    }

    #[test]
    fn test_extract_cpgs_spans_buffer_chunks() {
        let extractor = CpGExtractor::new("test.fa".to_string());
        let fasta = b">chr1\nAACG";

        let cpg_data = extractor
            .extract_from_reader(BufReader::with_capacity(3, &fasta[..]))
            .expect("extract CpGs across a buffer boundary");

        assert_eq!(cpg_data.cpgs.len(), 1);
        assert_eq!(cpg_data.cpgs[0].start, 2);
    }
}
