# methx

**A Rust methylation processor that converts Bismark coverage into versioned HDF5 and QC outputs.**

`methx` extracts CpG coordinates from reference FASTA files, processes Bismark coverage files with bounded parallelism, writes a documented custom HDF5 schema, and generates coverage and annotation reports. An included R function converts that custom file into a native Methrix HDF5 directory when standard Methrix loading is required.

## What it does

- Extracts CpG universes from FASTA or pinned built-in genome names.
- Processes `.bismark.cov.gz` inputs into beta and coverage assays.
- Writes transactional HDF5, QC workbook, and optional GTF annotation outputs.
- Supports `hg19`, `hg38`, `mm10`, and `mm39` genome downloads behind the `download` feature.
- Provides direct `rhdf5` access and an explicit custom-HDF5 → native Methrix conversion path.

## Install

`methx` uses HDF5. A release binary is the easiest path; build from source when developing:

```bash
git clone https://github.com/rainoffallingstar/methx.git
cd methx
cargo build --release
./target/release/methx --help
```

For local HDF5 development, set the environment variables required by your platform or use the repository's `rust_build` environment. See [HDF5 dependency notes](docs/HDF5_DEPENDENCY.md).

## Quick start

Extract a CpG universe once:

```bash
methx extract-cpgs \
  --genome hg38.fa \
  --output hg38_cpgs.ron
```

Process Bismark coverage:

```bash
methx process \
  --input bismark_output \
  --output results \
  --genome hg38_cpgs.ron \
  --threads 8
```

Generate a coverage-only report from an existing output:

```bash
methx qc-report \
  --input results \
  --output results/qc_report.xlsx
```

Download a pinned built-in genome when the optional feature is enabled:

```bash
cargo build --release --features download
methx download-genome --genome hg38 --output genomes
```

## Main commands

| Command | Purpose |
| --- | --- |
| `extract-cpgs` | Build a RON CpG coordinate file from FASTA or a built-in genome. |
| `process` | Convert Bismark coverage into HDF5, QC, and optional annotation outputs. |
| `qc-report` | Regenerate a coverage-only Excel or report output. |
| `download-genome` | Stream, verify, and atomically install a built-in genome release. |

Important `process` options include `--min-coverage`, `--remove-uncovered`, `--annotation-dir`, `--skip-annotation`, and `--threads`.

## HDF5 contract

The primary custom output is `assays.h5` (with `methrix_data.h5` as an identical filename alias):

```text
assays.h5
├── beta
├── cov
├── rowData/{chr,seqnames,start,end,width,strand}
├── colData/{sample_id,sample_name}
└── metadata/{genome,is_h5,schema_name,schema_version,loader_compatibility}
```

Coordinates are 1-based closed intervals. The custom file is directly readable with `rhdf5`, but it is **not** itself a native `methrix::load_HDF5_methrix()` directory.

## Native Methrix interoperability

Source the standalone exporter:

```r
source("scripts/export_methrix_hdf5.R")

export_methx_h5_to_methrix(
  methx_h5_path = "results/assays.h5",
  output_directory = "results/methrix_h5",
  validate = TRUE
)

methrix_object <- methrix::load_HDF5_methrix("results/methrix_h5")
```

The exporter uses `methrix::save_HDF5_methrix()` and, by default, reloads the result to validate coordinates, sample metadata, coverage, beta values, and the uncovered-value mask. Required R packages and details are documented in [the R compatibility guide](docs/R_METHRIX_COMPATIBILITY_GUIDE.md).

## Outputs and annotation

- HDF5 assay file with beta and coverage matrices.
- Coverage QC workbook from `qc-report` or `process`.
- `CpG_annotation_report.xlsx` with bounded summary categories.
- `CpG_annotation_details.tsv.gz` with unbounded per-CpG annotation detail when annotation is enabled.

Outputs are staged and published transactionally after native schema validation.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
```

## Documentation

- [Documentation index](docs/INDEX.md)
- [Build guide](docs/BUILD.md)
- [HDF5 structure](docs/HDF5_STRUCTURE_AND_COORDINATES.md)
- [R/Methrix compatibility](docs/R_METHRIX_COMPATIBILITY_GUIDE.md)
- [Testing quick reference](docs/TESTING_QUICK_REF.md)

## License and repository

MIT · [rainoffallingstar/methx](https://github.com/rainoffallingstar/methx)
