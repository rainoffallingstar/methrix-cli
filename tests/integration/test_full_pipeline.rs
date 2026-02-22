use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_full_pipeline() {
    // This test requires actual data files
    // For now, we'll test the individual components

    // Test CpG extraction
    let extractor = crate::genome::cpg::CpGExtractor::new("test_data/hg19.fa".to_string());
    // ... (would need test data)
}

#[test]
fn test_bismark_reader() {
    use crate::bismark::BismarkReader;

    let reader = BismarkReader::new("test_data/sample.bismark.cov.gz".to_string());
    // ... (would need test data)
}

#[test]
fn test_h5_compatibility() {
    // This test verifies H5 file can be loaded by R
    // After running the pipeline, test with:
    // library(methrix)
    // m <- load_HDF5_methrix("tests/output/methrix_data.h5")
    // print(m)
}
