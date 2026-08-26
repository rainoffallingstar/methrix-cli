# methx

High-performance methylation data processor - Bismark to HDF5 conversion tool.

> **Note**: This project is based on the [R methrix package](https://github.com/CompEpigen/methrix).

## Features

- **Fast CpG extraction** from reference genome FASTA files
- **Efficient Bismark file processing** with parallel support
- **Versioned custom HDF5 output** for direct access with R's `rhdf5`
- **Quality control reports** in Excel format
- **Cross-platform** standalone binary

## Installation

### Build from source

```bash
# Clone the repository
git clone https://github.com/rainoffallingstar/methx.git
cd methx

# Build release version
cargo build --release

# The binary will be at: target/release/methx
```

### Requirements

- Rust 1.75 or later
- HDF5 libraries (see below)

#### Installing HDF5

**Ubuntu/Debian:**
```bash
sudo apt-get install libhdf5-dev
```

**macOS:**
```bash
brew install hdf5
```

**Windows:**
- Install HDF5 from https://www.hdfgroup.org/downloads/index.html
- Set HDF5_DIR environment variable

## Usage

### Basic workflow

```bash
# 1. Extract CpG sites from reference genome (optional one-time step)
methx extract-cpgs \
  --genome hg19.fa \
  --output hg19_cpgs.ron

# 2. Process Bismark output files
methx process \
  --input bismark_output/ \
  --output results/ \
  --genome hg19_cpgs.ron \
  --threads 8

# 3. Generate QC report (can be run separately)
methx qc-report \
  --input results/ \
  --output qc_report.xlsx
```

### Download reference genomes

The download command requires `--features download`. Built-in releases (`hg19`, `hg38`, `mm10`, and `mm39`) are pinned to UCSC HTTPS URLs and official compressed-source MD5 values. Downloads are streamed with timeout and size limits, verified before publication, and atomically installed as `<release>.fa` together with `<release>.fa.provenance.ron`. Cached FASTA files are reused only after their provenance, byte size, and FASTA MD5 are revalidated.

```bash
# Download built-in genome (hg19, hg38, mm10, mm39)
methx download-genome \
  --genome hg19 \
  --output genomes/
```

### Commands

#### `methx process`

Process Bismark output files into methrix format.

```bash
methx process [OPTIONS]

Options:
  -i, --input <DIR>           Input directory with .bismark.cov.gz files
  -o, --output <DIR>          Output directory for H5 files
  -g, --genome <GENOME>       Reference genome (FASTA or pre-extracted .ron)
  -t, --threads <N>           Number of threads [default: CPU count]
      --min-coverage <N>       Minimum coverage [default: 1]
      --remove-uncovered      Remove uncovered loci [default: true]
      --annotation-dir <DIR>  Annotation resources directory (e.g. hg19.gtf or hg19.gtf.gz)
      --skip-annotation       Skip CpG annotation report generation
  -v, --verbose               Enable verbose logging
```

#### `methx extract-cpgs`

Extract CpG sites from reference genome.

```bash
methx extract-cpgs [OPTIONS]

Options:
  -g, --genome <GENOME>       Genome FASTA file or built-in name
  -o, --output <FILE>         Output RON file for CpG data
      --contigs <CONTIGS>     Contigs to include [default: autosomes + sex chromosomes]
  -v, --verbose               Enable verbose logging
```

#### `methx download-genome`

Download reference genome from UCSC.

```bash
methx download-genome [OPTIONS]

Options:
  -g, --genome <GENOME>       Genome name (hg19, hg38, mm10, mm39)
  -o, --output <DIR>          Output directory
  -v, --verbose               Enable verbose logging
```

#### `methx qc-report`

Regenerate a coverage-only QC report from an existing methrix H5 object.

```bash
methx qc-report [OPTIONS]

Options:
  -i, --input <DIR>           Input directory with methrix H5 object
  -o, --output <FILE>         Output Excel file
  -v, --verbose               Enable verbose logging
```

## Output files

### HDF5 file structure

The generated H5 file uses the versioned `methx.custom-hdf5` schema. It is designed for direct dataset access with R's `rhdf5`; it is **not** a standard `saveHDF5SummarizedExperiment()` directory and is not currently loadable through `HDF5Array::loadHDF5SummarizedExperiment()` or `methrix::load_HDF5_methrix()`.

`assays.h5` is the primary file. `methrix_data.h5` is an identical filename alias, not a different compatibility format:

```
methrix_data.h5
├── beta              # Methylation matrix (f32)
├── cov               # Coverage matrix (u32)
├── rowData/
│   ├── chr           # Chromosome
│   ├── seqnames      # Chromosome alias for direct R access
│   ├── start         # Start position (1-based, closed)
│   ├── end           # End position (1-based, closed)
│   ├── width         # Closed interval width
│   └── strand        # Strand (+/-/*)
├── colData/
│   ├── sample_id     # Sample names
│   └── sample_name   # Sample name alias
└── metadata/
    ├── genome                # Reference genome name
    ├── is_h5                 # HDF5 format flag
    ├── schema_name           # methx.custom-hdf5
    ├── schema_version        # Current schema version
    └── loader_compatibility  # Explicit supported/unsupported loader contract
```

### QC report

Excel file with coverage statistics:
- Total CpGs
- Covered CpGs
- Coverage distribution (1X, 2X, 3X, 4X, 5X, 10X)

### CpG annotation report

`methx process` publishes the annotation outputs as one transaction with HDF5 and QC outputs:

- `CpG_annotation_report.xlsx`: bounded `ChIPseeker_By_Sample` summary data. The required qctb categories `Promoter`, `Exon`, `Intron`, and `Intergenic` are always present, including zero-count columns; additional categories follow in lexical order.
- `CpG_annotation_details.tsv.gz`: unbounded per-CpG GTF annotation details with chromosome, 1-based closed coordinates, strand, annotation, gene, transcript, TSS distance, and exon/intron rank.

The annotation query path builds chromosome-local sorted interval buckets with prefix maximum end coordinates, then uses a start-coordinate binary search to avoid scanning intervals that cannot overlap the queried CpG. Queries are evaluated in parallel with Rayon, and the `--threads` value bounds the processing pool. The details table is intentionally not stored in Excel, so WGBS-sized datasets are not constrained by Excel's 1,048,576-row worksheet limit. `--skip-annotation` transactionally removes stale copies of both annotation outputs.

## R direct-schema integration

The generated H5 files can be read directly with `rhdf5`. The following example manually constructs a `SummarizedExperiment` in memory; this does not claim that the file itself satisfies the standard HDF5Array loader contract:

```r
library(rhdf5)

# Read data using new dataset names (beta/cov)
h5_file <- "results/assays.h5"
beta <- h5read(h5_file, "/beta")
cov <- h5read(h5_file, "/cov")

# Read coordinates
chr <- h5read(h5_file, "/rowData/chr")
start <- h5read(h5_file, "/rowData/start")
end <- h5read(h5_file, "/rowData/end")
strand <- h5read(h5_file, "/rowData/strand")

# Create SummarizedExperiment object
library(SummarizedExperiment)
library(GenomicRanges)

gr <- GRanges(chr, IRanges(start, end), strand)
coldata <- DataFrame(sample_id = h5read(h5_file, "/colData/sample_id"))

se <- SummarizedExperiment(
  assays = list(beta = beta, cov = cov),
  rowRanges = gr,
  colData = coldata
)

# Use with most Bioconductor functions
assay(se, "beta")
assay(se, "cov")
```

**Loader status**: `HDF5Array` and `methrix` are not available in the current validation environment, and the custom schema intentionally does not claim compatibility with their standard loaders. No synthetic `se.rds` is generated.

For methrix package workflows, convert through an explicitly supported interchange format and validate that downstream path independently.

## Performance

The assay writer stores sample-by-CpG HDF5 datasets with explicit chunking and writes bounded CpG blocks instead of materializing a full transposed copy. The processing pool limits concurrent per-sample temporary vectors to `--threads`, and HDF5, QC, and annotation outputs are staged and published as one rollback-capable transaction after native schema validation.

## Development

### Run tests

```bash
# Unit and real minimal CLI integration tests
cargo test --all-targets --all-features --locked

# Benchmarks
cargo bench
```

### Project structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── cli/                 # Command implementations
├── genome/              # Reference genome handling
├── bismark/             # Bismark file processing
├── processing/          # Core processing logic
├── hdf5/                # HDF5 I/O
└── qc/                  # Quality control
```

## License

MIT License - see LICENSE file for details.
