pub mod cpg;
pub mod download;

pub use cpg::{extract_and_save, load_cpg_data, ContigInfo, CpGData, CpGExtractor, CpGSite};
pub use download::download_genome;
