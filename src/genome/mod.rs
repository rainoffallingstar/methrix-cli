pub mod cpg;
pub mod download;

pub use cpg::{extract_and_save, load_cpg_data, ContigInfo, CpGData, CpGExtractor, CpGSite};
#[cfg(feature = "download")]
pub use download::download_genome;
