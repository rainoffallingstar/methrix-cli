use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::bismark::BismarkReader;
use crate::genome::cpg::{CpGData, CpGSite};
use crate::hdf5::se_compat::SummarizedExperimentWriter;
use crate::processing::{filter, stats};

pub struct MethrixProcessor {
    cpg_data: CpGData,
    cpg_index: HashMap<(String, u32), usize>,
}

impl MethrixProcessor {
    pub fn new(cpg_data: CpGData) -> Result<Self> {
        let mut cpg_index = HashMap::new();
        for (cpg_index_value, cpg) in cpg_data.cpgs.iter().enumerate() {
            let key = (canonical_contig_name(&cpg.chr), cpg.start);
            if let Some(existing_index) = cpg_index.insert(key.clone(), cpg_index_value) {
                bail!(
                    "Reference CpG data contains duplicate canonical key {}:{} at indices {} and {}",
                    key.0,
                    key.1,
                    existing_index,
                    cpg_index_value
                );
            }
        }

        Ok(Self {
            cpg_data,
            cpg_index,
        })
    }

    pub fn process_files_parallel(
        &self,
        files: Vec<String>,
        n_threads: usize,
        min_coverage: u32,
        remove_uncovered: bool,
    ) -> Result<MethrixData> {
        if files.is_empty() {
            bail!("No Bismark coverage files were provided");
        }
        if n_threads == 0 {
            bail!("Thread count must be at least 1");
        }

        let sample_names = normalized_sample_names(&files)?;
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(n_threads)
            .build()
            .context("Failed to create thread pool")?;

        let n_cpgs = self.cpg_data.cpgs.len();
        let n_samples = files.len();
        let mut beta_matrix = ndarray::Array2::<f32>::from_elem((n_cpgs, n_samples), f32::NAN);
        let mut cov_matrix = ndarray::Array2::<u32>::zeros((n_cpgs, n_samples));

        let results: Vec<(usize, ProcessedSample)> = pool
            .install(|| {
                files
                    .par_iter()
                    .enumerate()
                    .map(|(sample_index, file_path)| {
                        let sample = self
                            .process_single_file(file_path, min_coverage)
                            .with_context(|| format!("Failed to process {}", file_path))?;
                        Ok::<(usize, ProcessedSample), anyhow::Error>((sample_index, sample))
                    })
                    .collect::<Vec<Result<(usize, ProcessedSample)>>>()
            })
            .into_iter()
            .collect::<Result<Vec<(usize, ProcessedSample)>>>()?;

        for (sample_index, sample) in results {
            beta_matrix
                .column_mut(sample_index)
                .assign(&ndarray::Array1::from(sample.beta_values));
            cov_matrix
                .column_mut(sample_index)
                .assign(&ndarray::Array1::from(sample.coverage_values));
        }

        let (beta_matrix, cov_matrix, covered_indices) = if remove_uncovered {
            filter::remove_uncovered(beta_matrix, cov_matrix)?
        } else {
            let indices = (0..self.cpg_data.cpgs.len()).collect();
            (beta_matrix, cov_matrix, indices)
        };

        let cpg_locations = covered_indices
            .iter()
            .map(|&index| self.cpg_data.cpgs[index].clone())
            .collect();

        Ok(MethrixData {
            beta_matrix,
            cov_matrix,
            cpg_locations,
            sample_names,
            genome: self.cpg_data.release_name.clone(),
        })
    }

    fn process_single_file(&self, file_path: &str, min_coverage: u32) -> Result<ProcessedSample> {
        let records = BismarkReader::new(file_path.to_string()).read()?;
        let n_cpgs = self.cpg_data.cpgs.len();
        let mut beta_values = vec![f32::NAN; n_cpgs];
        let mut coverage_values = vec![0u32; n_cpgs];
        let mut seen_keys = HashSet::with_capacity(records.len());
        let mut matched_records = 0usize;
        let mut input_contigs = HashSet::new();
        let mut matched_contigs = HashSet::new();

        for record in records {
            let canonical_contig = canonical_contig_name(&record.chr);
            input_contigs.insert(canonical_contig.clone());
            let key = (canonical_contig, record.start);
            if !seen_keys.insert(key.clone()) {
                bail!(
                    "Duplicate CpG record in {} at canonical position {}:{}; aggregate duplicate counts upstream or remove duplicate rows",
                    file_path,
                    key.0,
                    key.1
                );
            }

            if let Some(&reference_index) = self.cpg_index.get(&key) {
                matched_records += 1;
                matched_contigs.insert(key.0.clone());
                let total_reads = record.total_reads()?;
                if total_reads >= min_coverage && total_reads > 0 {
                    coverage_values[reference_index] = total_reads;
                    beta_values[reference_index] = record
                        .beta_value()?
                        .context("Non-zero coverage unexpectedly produced no beta value")?;
                }
            }
        }

        if matched_records == 0 {
            bail!(
                "No Bismark CpG records in {} matched the reference CpG index. Input contigs: {:?}; check reference build and chromosome naming",
                file_path,
                sorted_values(input_contigs)
            );
        }

        let unmatched_contigs: Vec<String> = input_contigs
            .difference(&matched_contigs)
            .cloned()
            .collect();
        if !unmatched_contigs.is_empty() {
            eprintln!(
                "warning: {} contains contigs with no matching reference CpGs: {:?}",
                file_path,
                sorted_values(unmatched_contigs.into_iter().collect())
            );
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
    pub cov_matrix: ndarray::Array2<u32>,
    pub cpg_locations: Vec<CpGSite>,
    pub sample_names: Vec<String>,
    pub genome: String,
}

struct ProcessedSample {
    beta_values: Vec<f32>,
    coverage_values: Vec<u32>,
}

pub(crate) fn canonical_contig_name(contig: &str) -> String {
    let trimmed_contig = contig.trim();
    let without_prefix = trimmed_contig
        .strip_prefix("chr")
        .or_else(|| trimmed_contig.strip_prefix("CHR"))
        .unwrap_or(trimmed_contig);
    let uppercase_name = without_prefix.to_ascii_uppercase();
    if uppercase_name == "M" {
        "MT".to_string()
    } else {
        uppercase_name
    }
}

fn normalized_sample_names(files: &[String]) -> Result<Vec<String>> {
    let mut names = Vec::with_capacity(files.len());
    let mut seen_names = HashSet::with_capacity(files.len());
    for file_path in files {
        let sample_name = extract_sample_name(file_path)?;
        if !seen_names.insert(sample_name.clone()) {
            bail!(
                "Multiple Bismark files normalize to the same sample ID {:?}",
                sample_name
            );
        }
        names.push(sample_name);
    }
    Ok(names)
}

fn extract_sample_name(file_path: &str) -> Result<String> {
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("Invalid Bismark file name {}", file_path))?;
    const SUFFIXES: [&str; 4] = [".bismark.cov.gz", ".bismark.cov", ".cov.gz", ".cov"];
    let sample_name = SUFFIXES
        .iter()
        .find_map(|suffix| file_name.strip_suffix(suffix))
        .with_context(|| format!("Unsupported Bismark coverage file suffix: {}", file_name))?;
    if sample_name.is_empty() {
        bail!("Bismark file {} produces an empty sample ID", file_name);
    }
    Ok(sample_name.to_string())
}

fn sorted_values(values: HashSet<String>) -> Vec<String> {
    let mut values: Vec<String> = values.into_iter().collect();
    values.sort();
    values
}

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub input_dir: String,
    pub output_dir: String,
    pub genome: String,
    pub threads: usize,
    pub min_coverage: u32,
    pub remove_uncovered: bool,
    pub annotation_dir: Option<String>,
    pub skip_annotation: bool,
}

pub fn run_pipeline(config: PipelineConfig) -> Result<()> {
    let PipelineConfig {
        input_dir,
        output_dir,
        genome,
        threads,
        min_coverage,
        remove_uncovered,
        annotation_dir,
        skip_annotation,
    } = config;

    fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    let genome_lowercase = genome.to_ascii_lowercase();
    let cpg_data = if genome_lowercase.ends_with(".ron") {
        println!("Loading pre-extracted CpG data from: {}", genome);
        crate::genome::cpg::load_cpg_data(&genome)?
    } else if Path::new(&genome).exists() {
        if is_fasta_input(&genome) {
            println!("Extracting CpG sites from FASTA: {}", genome);
            crate::genome::cpg::CpGExtractor::new(genome.clone()).extract()?
        } else {
            println!("Loading CpG data from: {}", genome);
            crate::genome::cpg::load_cpg_data(&genome)?
        }
    } else {
        let ron_path = format!("{}.ron", genome);
        if Path::new(&ron_path).exists() {
            println!("Loading pre-extracted CpG data from: {}", ron_path);
            crate::genome::cpg::load_cpg_data(&ron_path)?
        } else {
            bail!(
                "CpG data not found for {}. Provide a FASTA file or pre-extracted RON file",
                genome
            );
        }
    };

    let bismark_files = find_bismark_files(&input_dir)?;
    println!("Found {} Bismark files", bismark_files.len());

    let processor = MethrixProcessor::new(cpg_data)?;
    let methrix_data =
        processor.process_files_parallel(bismark_files, threads, min_coverage, remove_uncovered)?;

    let assays_h5_path = Path::new(&output_dir).join("assays.h5");
    let compat_h5_path = Path::new(&output_dir).join("methrix_data.h5");
    println!("Writing HDF5 file to: {}", assays_h5_path.display());

    SummarizedExperimentWriter::new(assays_h5_path.to_string_lossy().to_string())
        .write_methrix_object(&methrix_data)?;
    fs::copy(&assays_h5_path, &compat_h5_path).with_context(|| {
        format!(
            "Failed to create compatibility HDF5 copy: {}",
            compat_h5_path.display()
        )
    })?;

    let qc_path = Path::new(&output_dir).join("CpG_coverage.xlsx");
    println!("Generating QC report: {}", qc_path.display());
    let sample_stats =
        stats::calculate_coverage_stats(&methrix_data.cov_matrix, &methrix_data.sample_names);
    crate::qc::report::generate_coverage_report(qc_path.to_str().unwrap(), &sample_stats)?;

    if skip_annotation {
        println!("Skipping CpG annotation report (--skip-annotation)");
    } else {
        let annotation_path = Path::new(&output_dir).join("CpG_annotation_report.xlsx");
        println!(
            "Generating CpG annotation report: {}",
            annotation_path.display()
        );
        let annotation_result = crate::annotation::annotate_cpgs(
            &methrix_data.cpg_locations,
            &methrix_data.cov_matrix,
            &methrix_data.sample_names,
            &methrix_data.genome,
            annotation_dir.as_deref(),
        )?;
        annotation_result.write_excel_report(annotation_path.to_str().unwrap())?;
    }

    println!("\nPipeline completed successfully!");
    Ok(())
}

fn is_fasta_input(path: &str) -> bool {
    let path_lowercase = path.to_ascii_lowercase();
    path_lowercase.ends_with(".fa")
        || path_lowercase.ends_with(".fasta")
        || path_lowercase.ends_with(".fna")
        || path_lowercase.ends_with(".fa.gz")
        || path_lowercase.ends_with(".fasta.gz")
        || path_lowercase.ends_with(".fna.gz")
}

fn find_bismark_files(dir: &str) -> Result<Vec<String>> {
    let path = Path::new(dir);
    let mut files = Vec::new();

    for entry in fs::read_dir(path).context("Failed to read input directory")? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_string = file_name.to_string_lossy();
        if file_name_string.ends_with(".bismark.cov.gz")
            || file_name_string.ends_with(".bismark.cov")
            || file_name_string.ends_with(".cov.gz")
            || file_name_string.ends_with(".cov")
        {
            files.push(entry.path().to_string_lossy().to_string());
        }
    }

    files.sort();
    if files.is_empty() {
        bail!("No Bismark coverage files found in directory: {}", dir);
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_common_contig_aliases() {
        assert_eq!(canonical_contig_name("chr1"), "1");
        assert_eq!(canonical_contig_name("1"), "1");
        assert_eq!(canonical_contig_name("chrM"), "MT");
        assert_eq!(canonical_contig_name("MT"), "MT");
    }

    #[test]
    fn normalizes_bismark_sample_suffixes() {
        assert_eq!(
            extract_sample_name("/tmp/sample.bismark.cov.gz").unwrap(),
            "sample"
        );
        assert_eq!(extract_sample_name("/tmp/sample.cov").unwrap(), "sample");
    }

    #[test]
    fn runs_complete_pipeline_with_temporary_fixture() {
        use crate::genome::cpg::{ContigInfo, CpGData, CpGSite};
        use hdf5::types::VarLenUnicode;
        use tempfile::tempdir;

        let temporary_directory = tempdir().unwrap();
        let input_directory = temporary_directory.path().join("input");
        let output_directory = temporary_directory.path().join("output");
        let annotation_directory = temporary_directory.path().join("annotations");
        fs::create_dir_all(&input_directory).unwrap();
        fs::create_dir_all(&annotation_directory).unwrap();

        let cpg_data = CpGData {
            cpgs: vec![
                CpGSite {
                    chr: "chr1".to_string(),
                    start: 9,
                    end: 11,
                    strand: '+',
                },
                CpGSite {
                    chr: "chr1".to_string(),
                    start: 19,
                    end: 21,
                    strand: '+',
                },
                CpGSite {
                    chr: "chr1".to_string(),
                    start: 29,
                    end: 31,
                    strand: '+',
                },
            ],
            contig_lens: vec![ContigInfo {
                contig: "chr1".to_string(),
                length: 100,
            }],
            release_name: "mini".to_string(),
        };
        let genome_path = temporary_directory.path().join("mini.ron");
        fs::write(&genome_path, ron::ser::to_string(&cpg_data).unwrap()).unwrap();

        fs::write(
            input_directory.join("sample_a.cov"),
            "chr1\t10\t10\t50.000000\t35000\t35000\nchr1\t20\t20\t100.000000\t5\t0\n",
        )
        .unwrap();
        fs::write(
            input_directory.join("sample_b.cov"),
            "1\t20\t20\t0.000000\t0\t5\n1\t30\t30\t25.000000\t1\t3\n",
        )
        .unwrap();
        fs::write(
            annotation_directory.join("mini.gtf"),
            concat!(
                "chr1\ttest\ttranscript\t1\t40\t.\t+\t.\tgene_id \"g1\"; transcript_id \"tx1\"; gene_name \"G1\";\n",
                "chr1\ttest\texon\t15\t25\t.\t+\t.\tgene_id \"g1\"; transcript_id \"tx1\"; gene_name \"G1\";\n"
            ),
        )
        .unwrap();

        run_pipeline(PipelineConfig {
            input_dir: input_directory.to_string_lossy().into_owned(),
            output_dir: output_directory.to_string_lossy().into_owned(),
            genome: genome_path.to_string_lossy().into_owned(),
            threads: 2,
            min_coverage: 1,
            remove_uncovered: true,
            annotation_dir: Some(annotation_directory.to_string_lossy().into_owned()),
            skip_annotation: false,
        })
        .unwrap();

        let assays_path = output_directory.join("assays.h5");
        let compatibility_path = output_directory.join("methrix_data.h5");
        assert!(assays_path.is_file());
        assert!(compatibility_path.is_file());
        assert!(output_directory.join("CpG_coverage.xlsx").is_file());
        assert!(output_directory
            .join("CpG_annotation_report.xlsx")
            .is_file());

        let file = hdf5::File::open(assays_path).unwrap();
        let coverage_dataset = file.dataset("cov").unwrap();
        assert_eq!(coverage_dataset.shape(), vec![2, 3]);
        assert_eq!(
            coverage_dataset.read_raw::<u32>().unwrap(),
            vec![70_000, 5, 0, 0, 5, 4]
        );
        assert_eq!(
            file.group("rowData")
                .unwrap()
                .dataset("start")
                .unwrap()
                .read_raw::<u32>()
                .unwrap(),
            vec![10, 20, 30]
        );
        assert_eq!(
            file.group("colData")
                .unwrap()
                .dataset("sample_name")
                .unwrap()
                .read_raw::<VarLenUnicode>()
                .unwrap()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["sample_a", "sample_b"]
        );
    }

    #[test]
    fn rejects_duplicate_normalized_sample_ids() {
        let files = vec!["sample.cov".to_string(), "sample.cov.gz".to_string()];
        assert!(normalized_sample_names(&files).is_err());
    }
}
