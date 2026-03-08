use anyhow::{Context, Result};
use needletail::parse_fastx_file;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};

trait SeparatedString {
    fn separated_string(&self) -> String;
}

impl SeparatedString for usize {
    fn separated_string(&self) -> String {
        let s = self.to_string();
        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 {
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
        let mut reader = parse_fastx_file(&self.fasta_path).context("Failed to open FASTA file")?;

        let mut all_cpgs = Vec::new();
        let mut contig_info = Vec::new();
        let mut total_cpgs = 0;

        // Read each chromosome
        while let Some(record_result) = reader.next() {
            let record = record_result
                .map_err(|e| anyhow::anyhow!("Failed to read FASTA record: {}", e))
                .context("Failed to read FASTA record")?;
            let chr = std::str::from_utf8(record.id())
                .context("Invalid UTF-8 in sequence ID")?
                .to_string();
            let seq = record.seq();

            // Check if we should include this contig
            if !self.should_include_chr(&chr) {
                continue;
            }

            // Record contig info
            contig_info.push(ContigInfo {
                contig: chr.clone(),
                length: seq.len() as u32,
            });

            // Extract CpG sites - equivalent to Biostrings::matchPattern("CG", sequence)
            let cpgs = self.extract_cpgs_from_sequence(&chr, &seq);
            total_cpgs += cpgs.len();
            all_cpgs.extend(cpgs);
        }

        println!(
            "-Done. Extracted {} CpGs from {} contigs.",
            total_cpgs.separated_string(),
            contig_info.len()
        );

        Ok(CpGData {
            cpgs: all_cpgs,
            contig_lens: contig_info,
            release_name: self.extract_genome_name()?,
        })
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
        // Handle with/without chr prefix
        let chr_clean = chr.strip_prefix("chr").unwrap_or(chr);

        matches!(
            chr_clean,
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
        )
    }

    /// Extract CpGs from sequence - equivalent to Biostrings::matchPattern("CG", ...)
    fn extract_cpgs_from_sequence(&self, chr: &str, seq: &[u8]) -> Vec<CpGSite> {
        let mut cpgs = Vec::new();
        let bytes = seq;

        // Find all CG patterns (positive strand)
        let mut i = 0;
        while i < bytes.len().saturating_sub(1) {
            // CpG is defined as C followed by G (on positive strand)
            if bytes[i] == b'C' && bytes[i + 1] == b'G' {
                cpgs.push(CpGSite {
                    chr: chr.to_string(),
                    start: i as u32, // 0-based, internal use
                    end: (i + 2) as u32,
                    strand: '+',
                });
            }
            i += 1;
        }

        cpgs
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

        let mut file = BufWriter::new(File::create(output_path)?);
        file.write_all(ron_string.as_bytes())?;

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
        assert!(extractor.is_standard_chromosome("chrX"));
        assert!(extractor.is_standard_chromosome("chrY"));
        assert!(!extractor.is_standard_chromosome("chrM"));
        assert!(!extractor.is_standard_chromosome("chrUn"));
    }

    #[test]
    fn test_extract_cpgs_from_sequence() {
        let extractor = CpGExtractor::new("test.fa".to_string());
        let seq = b"ATCGATCGAA";
        let cpgs = extractor.extract_cpgs_from_sequence("chr1", seq);

        // Should find 2 CG sites
        assert_eq!(cpgs.len(), 2);
        assert_eq!(cpgs[0].start, 2); // First CG at position 2
        assert_eq!(cpgs[1].start, 6); // Second CG at position 6
    }
}
