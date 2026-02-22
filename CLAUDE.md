# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Methrix CLI is a high-performance Rust command-line tool for processing Bismark bisulfite sequencing data into methrix-compatible HDF5 format. It serves as a complete Rust alternative to the original R implementation (methrix R package), achieving 5-10x performance improvements while maintaining 100% compatibility with the R package.

**Key constraint**: The generated HDF5 files MUST be compatible with R's `HDF5Array::loadHDF5SummarizedExperiment()` and methrix's `load_HDF5_methrix()`.

## Build and Development Commands

```bash
# Build
cargo build                  # Development build
cargo build --release        # Optimized release build (with LTO, strip, opt-level 3)
cargo clean                  # Clean build artifacts

# Testing
cargo test                   # Run all tests
cargo test --test '*'        # Run integration tests
cargo bench                  # Run benchmarks

# Code quality
cargo fmt                    # Format code
cargo clippy                 # Lint code
cargo audit                  # Security audit

# Running the tool
./target/release/methrix --help
./target/release/methrix process --input <dir> --output <dir> --genome <genome>
```

## System Dependencies

- **HDF5 libraries** (required):
  - Ubuntu/Debian: `apt-get install libhdf5-dev`
  - macOS: `brew install hdf5`
  - Windows: Install from https://www.hdfgroup.org/downloads/index.html

## High-Level Architecture

The project follows a **layered architecture** with clear module separation:

```
┌─────────────────────────────────────────────────┐
│         CLI Layer (main.rs, cli/)               │  ← User Interface
│  - Argument parsing (clap)                       │
│  - Command routing                               │
│  - Progress display (indicatif)                  │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│      Reference Genome Layer (genome/)            │  ← Data Preparation
│  - CpG extraction (ported from R::extract_CPGs)  │
│  - FASTA reading (needletail)                    │
│  - Genome download (reqwest)                     │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│      Data Processing Layer                       │  ← Core Logic
│    (bismark/, processing/)                       │
│  - Bismark file parsing (ported from R::read_bdg)│
│  - Parallel processing (rayon)                   │
│  - Data filtering                                │
│  - Statistics calculation                        │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│      Output Layer (hdf5/, qc/)                   │  ← Results
│  - HDF5 writing (SummarizedExperiment compatible)│
│  - QC report generation (Excel)                  │
└─────────────────────────────────────────────────┘
```

### Module Organization

```
src/
├── main.rs              # CLI entry point, command routing
├── lib.rs               # Library exports, re-exports
├── cli/                 # Command implementations
│   ├── process.rs       # Main processing pipeline
│   └── mod.rs
├── genome/              # Reference genome handling
│   ├── cpg.rs           # CpG extraction (CpGExtractor, CpGData, CpGSite)
│   ├── download.rs      # Genome downloads
│   └── mod.rs
├── bismark/             # Bismark file processing
│   ├── reader.rs        # File parser (BismarkRecord)
│   └── mod.rs
├── processing/          # Core processing logic
│   ├── pipeline.rs      # Main pipeline (MethrixProcessor, MethrixData)
│   ├── filter.rs        # Data filtering (remove_uncovered, coverage_filter)
│   ├── stats.rs         # Statistics (SampleStats)
│   └── mod.rs
├── hdf5/                # HDF5 I/O
│   ├── se_compat.rs     # SE-compatible writer
│   └── mod.rs
└── qc/                  # Quality control
    ├── report.rs        # Excel report generation
    └── mod.rs
```

## Core Data Structures

### Primary Types (defined in `src/genome/cpg.rs`)

```rust
// CpG site representation
pub struct CpGSite {
    pub chr: String,      // Chromosome
    pub start: u32,       // 0-based position
    pub end: u32,
    pub strand: char,
}

// Collection of CpG sites
pub struct CpGData {
    pub cpgs: Vec<CpGSite>,
    pub contig_lens: Vec<ContigInfo>,
    pub release_name: String,
}

// Bismark record (in src/bismark/reader.rs)
pub struct BismarkRecord {
    pub chr: String,
    pub start: u32,                // 0-based
    pub methylated_reads: u32,
    pub unmethylated_reads: u32,
}

// Sample statistics (in src/processing/stats.rs)
pub struct SampleStats {
    pub sample_name: String,
    pub n_covered: usize,
    pub n_total: usize,
    pub mean_coverage: f32,
    pub coverage_distribution: Vec<(u16, usize)>,
}
```

## Data Flow

1. **Input**: Bismark .cov.gz files → `BismarkReader` (converts 1-based to 0-based coordinates)
2. **Reference**: Genome FASTA → `CpGExtractor` → `CpGData` (serialized as .ron)
3. **Processing**: Parallel alignment to CpG sites → `MethrixProcessor`
4. **Filtering**: Remove uncovered loci → `filter::remove_uncovered()`
5. **Statistics**: Calculate coverage → `stats::calculate_coverage_stats()`
6. **Output**: HDF5 (R-compatible) + Excel QC report

## R Function Porting Map

The Rust implementation ports these specific R functions from the methrix package:
- `extract_CPGs()` → `CpGExtractor::extract()`
- `read_bdg()` → `BismarkReader::read()`
- `vect_code_batch()` → `MethrixProcessor::process_files_parallel()`
- `remove_uncovered()` → `filter::remove_uncovered()`
- `coverage_filter()` → `filter::coverage_filter()`
- `get_stats()` → `stats::calculate_coverage_stats()`

## HDF5 Output Structure (R Compatibility Critical)

The generated H5 file MUST be compatible with R's `HDF5Array::loadHDF5SummarizedExperiment()`:

```
methrix_data.h5
├── assays/
│   ├── beta          # f32 matrix (methylation values)
│   └── cov           # u16 matrix (coverage counts)
├── rowData/
│   ├── chr           # String array (chromosomes)
│   ├── start         # u32 array (0-based positions)
│   ├── end           # u32 array
│   └── strand        # String array (strands)
├── colData/
│   └── sample_id     # String array (sample names)
└── metadata/
    ├── genome        # Scalar (reference genome name)
    └── is_h5         # Scalar (HDF5 format flag)
```

**Key compatibility requirements**:
- Use HDF5 Group structure (assays/, rowData/, colData/, metadata/)
- Column-major storage order (R default)
- GZIP compression (level 6)
- SE-specific attributes (se_version, delayed_array_type)
- Data types: beta (f32), cov (u16)

## Performance Optimization Patterns

1. **Memory optimization**: Use `u16` instead of `u32` for coverage, `f32` instead of `f64` for methylation values
2. **Zero-copy**: Memory mapping for large files (`memmap2`)
3. **Data parallelism**: `rayon` for concurrent processing (configurable thread pools)
4. **I/O optimization**: Streaming file processing, batch HDF5 writes

## Key Design Patterns

1. **Builder Pattern**: `CpGExtractor` with fluent API (`.contigs()` method)
2. **Error Propagation**: Use `anyhow` for context-rich error handling with `.context()`
3. **Type Safety**: Strong typing with defined data structures (no `any` allowed)
4. **Module Separation**: Each major component is a separate module with clear exports

## Important Coordinate System

- **Bismark files**: 1-based coordinates (converted to 0-based internally)
- **Internal representation**: 0-based coordinates
- **HDF5 output**: 0-based coordinates (R bioconductor standard)
- When reading Bismark files, always subtract 1 from start position

## CLI Commands

```bash
# Main processing command
methrix process -i <input> -o <output> -g <genome> [OPTIONS]

# Extract CpG sites (optimization - one-time per genome)
methrix extract-cpgs -g <genome> -o <output.ron> [OPTIONS]

# Download reference genomes
methrix download-genome -g <hg19|hg38|mm10|mm39> -o <dir>

# Generate QC report
methrix qc-report -i <h5_dir> -o <output.xlsx>
```

## Testing Infrastructure

- Unit tests in individual source files
- Integration tests in `tests/integration/`
- R compatibility verification: `tests/integration/test_r_compatibility.R`
- Test data generator: `scripts/generate_test_data.py`

## Performance Characteristics

- **5-10x faster** than R implementation
- **30-50% less memory** usage
- **Sub-second startup** time
- Optimizations: memory mapping, parallel processing, efficient data types

## Type-First Development

This project follows Type-First development principles:
- All data structures are defined in their respective modules
- Module communication relies on defined interfaces
- No implicit type inference or untyped data structures
- Prefer defining types in module `mod.rs` or dedicated files

## Error Handling Pattern

Use `anyhow` for context-rich errors:

```rust
use anyhow::{Context, Result};

let file = File::open(&path)
    .context("Failed to open input file")?;
```
