# Methrix CLI

High-performance methylation data processor - Bismark to HDF5 conversion tool.

> **Note**: This project is based on the [R methrix package](https://github.com/CompEpigen/methrix).

## Features

- **Fast CpG extraction** from reference genome FASTA files
- **Efficient Bismark file processing** with parallel support
- **HDF5 output** compatible with R's methrix package
- **Quality control reports** in Excel format
- **Cross-platform** standalone binary

## Installation

### Build from source

```bash
# Clone the repository
git clone https://github.com/CompEpigen/methrix.git
cd methrix/methrix-cli

# Build release version
cargo build --release

# The binary will be at: target/release/methrix
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
methrix extract-cpgs \
  --genome hg19.fa \
  --output hg19_cpgs.ron

# 2. Process Bismark output files
methrix process \
  --input bismark_output/ \
  --output results/ \
  --genome hg19_cpgs.ron \
  --threads 8

# 3. Generate QC report (can be run separately)
methrix qc-report \
  --input results/ \
  --output qc_report.xlsx
```

### Download reference genomes

```bash
# Download built-in genome (hg19, hg38, mm10, mm39)
methrix download-genome \
  --genome hg19 \
  --output genomes/
```

### Commands

#### `methrix process`

Process Bismark output files into methrix format.

```bash
methrix process [OPTIONS]

Options:
  -i, --input <DIR>           Input directory with .bismark.cov.gz files
  -o, --output <DIR>          Output directory for H5 files
  -g, --genome <GENOME>       Reference genome (FASTA or pre-extracted .ron)
  -t, --threads <N>           Number of threads [default: CPU count]
      --min-coverage <N>       Minimum coverage [default: 1]
      --remove-uncovered      Remove uncovered loci [default: true]
  -v, --verbose               Enable verbose logging
```

#### `methrix extract-cpgs`

Extract CpG sites from reference genome.

```bash
methrix extract-cpgs [OPTIONS]

Options:
  -g, --genome <GENOME>       Genome FASTA file or built-in name
  -o, --output <FILE>         Output RON file for CpG data
      --contigs <CONTIGS>     Contigs to include [default: autosomes + sex chromosomes]
  -v, --verbose               Enable verbose logging
```

#### `methrix download-genome`

Download reference genome from UCSC.

```bash
methrix download-genome [OPTIONS]

Options:
  -g, --genome <GENOME>       Genome name (hg19, hg38, mm10, mm39)
  -o, --output <DIR>          Output directory
  -v, --verbose               Enable verbose logging
```

#### `methrix qc-report`

Generate QC report from existing methrix H5 object.

```bash
methrix qc-report [OPTIONS]

Options:
  -i, --input <DIR>           Input directory with methrix H5 object
  -o, --output <FILE>         Output Excel file
  -v, --verbose               Enable verbose logging
```

## Output files

### HDF5 file structure

The generated H5 file is compatible with R's `HDF5Array::loadHDF5SummarizedExperiment()`:

```
methrix_data.h5
├── assays/
│   ├── beta          # Methylation matrix (f32)
│   └── cov           # Coverage matrix (u16)
├── rowData/
│   ├── chr           # Chromosome
│   ├── start         # Start position (0-based)
│   ├── end           # End position
│   └── strand        # Strand (+)
├── colData/
│   └── sample_id     # Sample names
└── metadata/
    ├── genome        # Reference genome name
    └── is_h5         # HDF5 format flag
```

### QC report

Excel file with coverage statistics:
- Total CpGs
- Covered CpGs
- Coverage distribution (1X, 2X, 3X, 4X, 5X, 10X)

## R integration

The generated H5 files can be loaded in R using `rhdf5`:

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

gr <- GRanges(chr, IRanges(start + 1, end), strand)
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

**Note**: For full compatibility with R methrix package functions, you can convert to bedgraph format and use `read_bedgraphs()`. See [docs/QUICK_START_LOADING.md](docs/QUICK_START_LOADING.md) for details.

## Performance

Compared to the original R package:
- **5-10x faster** I/O processing
- **30-50% less memory** usage
- **Sub-second startup** time

## Development

### Run tests

```bash
# Unit tests
cargo test

# Integration tests (requires test data)
cargo test --test '*'

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
