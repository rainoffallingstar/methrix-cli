use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::bismark::{BismarkReader, BismarkRecord};
use crate::genome::cpg::{CpGData, CpGSite};
use crate::hdf5::se_compat::SummarizedExperimentWriter;
use crate::processing::{filter, stats};

pub struct MethrixProcessor {
    cpg_data: CpGData,
    cpg_index: HashMap<(String, u32), usize>,
}

impl MethrixProcessor {
    pub fn new(cpg_data: CpGData) -> Self {
        // Build CpG index - similar to R's data.table::setkey
        let mut cpg_index = HashMap::new();
        for (idx, cpg) in cpg_data.cpgs.iter().enumerate() {
            cpg_index.insert((cpg.chr.clone(), cpg.start), idx);
        }

        Self {
            cpg_data,
            cpg_index,
        }
    }

    /// Process multiple Bismark files in parallel - ported from R::vect_code_batch
    pub fn process_files_parallel(
        &self,
        files: Vec<String>,
        n_threads: usize,
        min_coverage: u16,
        remove_uncovered: bool,
    ) -> Result<MethrixData> {
        // Set up thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(n_threads)
            .build()
            .context("Failed to create thread pool")?;

        let n_cpgs = self.cpg_data.cpgs.len();
        let n_samples = files.len();

        // Initialize matrices
        let mut beta_matrix = ndarray::Array2::<f32>::zeros((n_cpgs, n_samples));
        let mut cov_matrix = ndarray::Array2::<u16>::zeros((n_cpgs, n_samples));

        // Process files in parallel
        let results: Vec<(usize, ProcessedSample)> = pool
            .install(|| {
                files
                    .par_iter()
                    .enumerate()
                    .map(|(sample_idx, file_path)| {
                        let sample = self
                            .process_single_file(file_path, min_coverage)
                            .with_context(|| format!("Failed to process {}", file_path))?;
                        Ok::<(usize, ProcessedSample), anyhow::Error>((sample_idx, sample))
                    })
                    .collect::<Vec<Result<(usize, ProcessedSample)>>>()
            })
            .into_iter()
            .collect::<Result<Vec<(usize, ProcessedSample)>>>()?;

        // Merge results
        for (sample_idx, sample) in results {
            beta_matrix
                .column_mut(sample_idx)
                .assign(&ndarray::Array1::from(sample.beta_values));
            cov_matrix
                .column_mut(sample_idx)
                .assign(&ndarray::Array1::from(sample.coverage_values));
        }

        // Apply filters
        let (beta_matrix, cov_matrix, covered_indices) = if remove_uncovered {
            filter::remove_uncovered(beta_matrix, cov_matrix)?
        } else {
            // If not filtering, keep all indices
            let indices = (0..self.cpg_data.cpgs.len()).collect();
            (beta_matrix, cov_matrix, indices)
        };

        // Filter cpg_locations to match filtered data
        let cpg_locations: Vec<crate::genome::cpg::CpGSite> = covered_indices
            .iter()
            .map(|&i| self.cpg_data.cpgs[i].clone())
            .collect();

        Ok(MethrixData {
            beta_matrix,
            cov_matrix,
            cpg_locations,
            sample_names: files.into_iter().map(|f| extract_sample_name(&f)).collect(),
            genome: self.cpg_data.release_name.clone(),
        })
    }

    fn process_single_file(&self, file_path: &str, min_coverage: u16) -> Result<ProcessedSample> {
        let reader = BismarkReader::new(file_path.to_string());
        let records = reader.read()?;
        let min_coverage = min_coverage as u32;

        let n_cpgs = self.cpg_data.cpgs.len();
        let mut beta_values = vec![f32::NAN; n_cpgs];
        let mut coverage_values = vec![0u16; n_cpgs];

        // Align to reference CpGs - ported from R::read_bdg
        for record in records {
            let chr = record.chr.clone();
            let start = record.start;
            if let Some(&idx) = self.cpg_index.get(&(chr, start)) {
                let total = record.total_reads();
                if total >= min_coverage && idx < n_cpgs {
                    coverage_values[idx] = total as u16;
                    beta_values[idx] = record.beta_value().unwrap();
                }
            }
        }

        Ok(ProcessedSample {
            beta_values,
            coverage_values,
        })
    }
}

#[derive(Debug)]
pub struct MethrixData {
    pub beta_matrix: ndarray::Array2<f32>,
    pub cov_matrix: ndarray::Array2<u16>,
    pub cpg_locations: Vec<CpGSite>,
    pub sample_names: Vec<String>,
    pub genome: String,
}

struct ProcessedSample {
    beta_values: Vec<f32>,
    coverage_values: Vec<u16>,
}

fn extract_sample_name(file_path: &str) -> String {
    let path = Path::new(file_path);
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Run the complete processing pipeline
pub fn run_pipeline(
    input_dir: String,
    output_dir: String,
    genome: String,
    threads: usize,
    min_coverage: u16,
    remove_uncovered: bool,
) -> Result<()> {
    // Create output directory
    fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    // Load or extract CpG data
    let genome_lc = genome.to_ascii_lowercase();
    let cpg_data = if genome_lc.ends_with(".ron") {
        // It's a pre-extracted RON file
        println!("Loading pre-extracted CpG data from: {}", genome);
        crate::genome::cpg::load_cpg_data(&genome)?
    } else if Path::new(&genome).exists() {
        // Check if it's actually a RON file or FASTA
        if is_fasta_input(&genome) {
            // It's a FASTA file, need to extract CpGs
            println!("Extracting CpG sites from FASTA: {}", genome);
            let extractor = crate::genome::cpg::CpGExtractor::new(genome.clone());
            extractor.extract()?
        } else {
            // Try to load as RON file
            println!("Loading CpG data from: {}", genome);
            crate::genome::cpg::load_cpg_data(&genome)?
        }
    } else {
        // Check if it's a pre-extracted RON file with .ron extension
        let ron_path = format!("{}.ron", genome);
        if Path::new(&ron_path).exists() {
            println!("Loading pre-extracted CpG data from: {}", ron_path);
            crate::genome::cpg::load_cpg_data(&ron_path)?
        } else {
            // Provide actionable local file guidance.
            println!("Genome file not found: {}", genome);
            println!("Please either:");
            println!("1. Provide a FASTA file path (.fa/.fasta/.fna, optional .gz)");
            println!("2. Provide a pre-extracted .ron file path");
            println!(
                "3. Run: methrix extract-cpgs --genome <fasta> --output {}.ron",
                genome
            );
            anyhow::bail!("CpG data not found")
        }
    };

    // Find Bismark files
    let bismark_files = find_bismark_files(&input_dir)?;
    println!("Found {} Bismark files", bismark_files.len());

    // Process files
    let processor = MethrixProcessor::new(cpg_data);
    let methrix_data =
        processor.process_files_parallel(bismark_files, threads, min_coverage, remove_uncovered)?;

    // Write H5 file
    let assays_h5_path = Path::new(&output_dir).join("assays.h5");
    let compat_h5_path = Path::new(&output_dir).join("methrix_data.h5");
    println!("Writing HDF5 file to: {}", assays_h5_path.display());

    let writer = SummarizedExperimentWriter::new(assays_h5_path.to_string_lossy().to_string());
    writer.write_methrix_object(&methrix_data)?;
    // Keep legacy filename for compatibility with older scripts.
    fs::copy(&assays_h5_path, &compat_h5_path).with_context(|| {
        format!(
            "Failed to create compatibility HDF5 copy: {}",
            compat_h5_path.display()
        )
    })?;

    // Generate QC report
    let qc_path = Path::new(&output_dir).join("CpG_coverage.xlsx");
    println!("Generating QC report: {}", qc_path.display());

    let sample_stats =
        stats::calculate_coverage_stats(&methrix_data.cov_matrix, &methrix_data.sample_names);
    crate::qc::report::generate_coverage_report(qc_path.to_str().unwrap(), &sample_stats)?;

    println!("\nPipeline completed successfully!");
    println!("Output files:");
    println!("  - HDF5: {}", assays_h5_path.display());
    println!("  - HDF5 (compat): {}", compat_h5_path.display());
    println!("  - QC Report: {}", qc_path.display());

    Ok(())
}

fn is_fasta_input(path: &str) -> bool {
    let path_lc = path.to_ascii_lowercase();
    path_lc.ends_with(".fa")
        || path_lc.ends_with(".fasta")
        || path_lc.ends_with(".fna")
        || path_lc.ends_with(".fa.gz")
        || path_lc.ends_with(".fasta.gz")
        || path_lc.ends_with(".fna.gz")
}

fn find_bismark_files(dir: &str) -> Result<Vec<String>> {
    let path = Path::new(dir);
    let mut files = Vec::new();

    for entry in fs::read_dir(path).context("Failed to read input directory")? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if file_name_str.ends_with(".bismark.cov.gz") || file_name_str.ends_with(".cov.gz") {
            files.push(entry.path().to_string_lossy().to_string());
        }
    }

    files.sort();

    if files.is_empty() {
        anyhow::bail!("No Bismark files found in directory: {}", dir);
    }

    Ok(files)
}
