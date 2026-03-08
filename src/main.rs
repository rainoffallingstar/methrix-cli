use clap::{Parser, Subcommand};
use methrix_cli::{cli::process, genome::cpg, qc::report};
use tracing::{info, Level};
use tracing_subscriber::fmt;

#[derive(Parser)]
#[command(name = "methrix-cli")]
#[command(about = "High-performance methylation data processor", long_about = None)]
#[command(version)]
#[command(author = "methrix contributors")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process Bismark output files into methrix format
    Process {
        /// Input directory containing Bismark .cov.gz files
        #[arg(short, long)]
        input: String,

        /// Output directory for H5 files and reports
        #[arg(short, long)]
        output: String,

        /// Reference genome path (.ron, .fa/.fasta/.fna, optional .gz)
        #[arg(short, long)]
        genome: String,

        /// Number of threads for parallel processing
        #[arg(short = 't', long, default_value_t = std::thread::available_parallelism().unwrap().get())]
        threads: usize,

        /// Minimum coverage for a CpG to be considered covered
        #[arg(long, default_value = "1")]
        min_coverage: u16,

        /// Remove loci uncovered across all samples
        #[arg(long, default_value = "true")]
        remove_uncovered: bool,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },

    /// Extract CpG sites from reference genome
    #[command(name = "extract-cpgs")]
    ExtractCpGs {
        /// Genome FASTA file path (.fa/.fasta/.fna, optional .gz)
        #[arg(short, long)]
        genome: String,

        /// Output file for CpG data (RON format)
        #[arg(short, long)]
        output: String,

        /// Contigs to include (default: autosomes + sex chromosomes)
        #[arg(long)]
        contigs: Option<Vec<String>>,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },

    /// Download reference genome from UCSC
    DownloadGenome {
        /// Genome name (hg19, hg38, mm10, mm39)
        #[arg(short, long)]
        genome: String,

        /// Output directory
        #[arg(short = 'o', long)]
        output: String,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },

    /// Generate QC report from existing methrix H5 object
    QCReport {
        /// Input directory containing methrix H5 object
        #[arg(short, long)]
        input: String,

        /// Output Excel file path
        #[arg(short, long)]
        output: String,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logger
    let log_level = if matches!(
        cli.command,
        Commands::Process { verbose: true, .. }
            | Commands::ExtractCpGs { verbose: true, .. }
            | Commands::DownloadGenome { verbose: true, .. }
            | Commands::QCReport { verbose: true, .. }
    ) {
        Level::DEBUG
    } else {
        Level::INFO
    };

    fmt().with_max_level(log_level).init();

    match cli.command {
        Commands::Process {
            input,
            output,
            genome,
            threads,
            min_coverage,
            remove_uncovered,
            ..
        } => {
            info!("Starting methrix processing pipeline");
            info!("Input: {}", input);
            info!("Output: {}", output);
            info!("Genome: {}", genome);
            info!("Threads: {}", threads);
            info!("Min coverage: {}", min_coverage);
            info!("Remove uncovered: {}", remove_uncovered);

            process::run_pipeline(
                input,
                output,
                genome,
                threads,
                min_coverage,
                remove_uncovered,
            )
        }

        Commands::ExtractCpGs {
            genome,
            output,
            contigs,
            ..
        } => {
            info!("Extracting CpG sites from genome: {}", genome);
            cpg::extract_and_save(genome, output, contigs)
        }

        Commands::DownloadGenome { genome, output, .. } => {
            info!("Downloading genome: {}", genome);
            #[cfg(feature = "download")]
            {
                methrix_cli::genome::download::download_genome(&genome, &output)
            }
            #[cfg(not(feature = "download"))]
            {
                anyhow::bail!(
                    "Download feature not enabled. Please rebuild with --features download"
                )
            }
        }

        Commands::QCReport { input, output, .. } => {
            info!("Generating QC report");
            report::generate_qc_report(&input, &output)
        }
    }
}
