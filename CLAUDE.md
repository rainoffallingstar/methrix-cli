# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

methx is a high-performance Rust command-line tool for processing Bismark bisulfite sequencing data into a versioned custom HDF5 schema. It is a Rust-native processing alternative inspired by the methrix R package, with direct R interoperability through `rhdf5` dataset access.

**Key constraint**: Generated files MUST satisfy `methx.custom-hdf5/1.0.0`, pass the Rust-native validator before publication, and truthfully identify their loader contract. They are not standard `saveHDF5SummarizedExperiment()` directories and MUST NOT be described as directly loadable by `HDF5Array::loadHDF5SummarizedExperiment()` or `methrix::load_HDF5_methrix()`.

## Build and Development Commands

```bash
# Build
cargo build                  # Development build
cargo build --release        # Optimized release build (with LTO, strip, opt-level 3)
cargo clean                  # Clean build artifacts

# Testing and quality gates
cargo fmt --all -- --check
cargo check --all-targets --all-features --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo build --all-targets --all-features --locked
cargo bench                  # Run benchmarks when performance changes

# Running the tool
./target/release/methx --help
./target/release/methx process --input <dir> --output <dir> --genome <genome>
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
│  - HDF5 writing (versioned custom schema)        │
│  - Native pre-publication schema validation       │
│  - QC and split annotation reports                │
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
│   ├── se_compat.rs     # Versioned custom HDF5 writer
│   ├── validate.rs      # Native schema/readback validator
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
6. **Output**: versioned custom HDF5 + Excel QC summary + Excel/TSV.gz annotation report set

## R Function Porting Map

The Rust implementation ports these specific R functions from the methrix package:
- `extract_CPGs()` → `CpGExtractor::extract()`
- `read_bdg()` → `BismarkReader::read()`
- `vect_code_batch()` → `MethrixProcessor::process_files_parallel()`
- `remove_uncovered()` → `filter::remove_uncovered()`
- `coverage_filter()` → `filter::coverage_filter()`
- `get_stats()` → `stats::calculate_coverage_stats()`

## HDF5 Output Structure

The primary `assays.h5` and identical `methrix_data.h5` alias use the versioned `methx.custom-hdf5/1.0.0` contract. Direct `rhdf5` access is supported; standard HDF5Array and methrix loaders are explicitly unsupported.

```
assays.h5
├── beta                      # chunked f32 [sample, CpG]
├── cov                       # chunked u32 [sample, CpG]
├── rowData/
│   ├── chr, seqnames         # chromosome strings
│   ├── start, end, width     # 1-based closed u32 coordinates
│   └── strand                # +, -, or *
├── colData/
│   ├── sample_id
│   └── sample_name
└── metadata/
    ├── genome
    ├── schema_name
    ├── schema_version
    ├── loader_compatibility
    └── is_h5
```

**Required behavior**:
- Keep beta as `f32` and coverage as `u32`.
- Keep assays chunked and satisfy `cov == 0` if and only if beta is NaN.
- Run `validate_custom_hdf5()` against staged files before publication.
- Do not introduce `se.rds` or claim compatibility without a real loader test and schema version change.

## Performance Optimization Patterns

1. **Bounded HDF5 writes**: write one sample/CpG block at a time without a full transposed assay copy.
2. **Bounded sample concurrency**: limit active per-sample temporary vectors with the configured Rayon pool.
3. **Single-pass statistics**: avoid full temporary columns and covered-value vectors.
4. **Transactional outputs**: stage, sync, validate, and publish the complete output set with rollback.

## Key Design Patterns

1. **Builder Pattern**: `CpGExtractor` with fluent API (`.contigs()` method)
2. **Error Propagation**: Use `anyhow` for context-rich error handling with `.context()`
3. **Type Safety**: Strong typing with defined data structures (no `any` allowed)
4. **Module Separation**: Each major component is a separate module with clear exports

## Important Coordinate System

- **Bismark files**: 1-based single-base coordinates; start is converted to 0-based internally.
- **Internal CpG representation**: 0-based start and end-exclusive interval.
- **HDF5 rowData**: 1-based closed `start`/`end` with checked `width = end - start + 1`.
- Bismark records with `end != start` are rejected.

## CLI Commands

```bash
# Main processing command
methx process -i <input> -o <output> -g <genome> [OPTIONS]

# Extract CpG sites (optimization - one-time per genome)
methx extract-cpgs -g <genome> -o <output.ron> [OPTIONS]

# Download reference genomes
methx download-genome -g <hg19|hg38|mm10|mm39> -o <dir>

# Generate QC report
methx qc-report -i <h5_dir> -o <output.xlsx>
```

## Testing Infrastructure

- Unit tests in individual source files
- Integration tests in `tests/integration/`
- R compatibility verification: `tests/integration/test_r_compatibility.R`
- Test data generator: `scripts/generate_test_data.py`

## Performance Characteristics

Do not repeat unverified speed or memory percentages. The enforced properties are bounded chunked HDF5 writing, a thread-bounded processing pool, single-pass coverage statistics, and rollback-capable publication. Benchmark representative RRBS/WGBS datasets before making quantitative performance claims.

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
