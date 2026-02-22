pub mod bismark;
pub mod cli;
pub mod genome;
pub mod hdf5;
pub mod processing;
pub mod qc;

// Re-exports for convenience
pub use genome::cpg::{load_cpg_data, ContigInfo, CpGData, CpGExtractor, CpGSite};
