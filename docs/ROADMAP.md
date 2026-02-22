# Methrix CLI Development Roadmap

## Version 0.1.0 (Current Release)

### Core Features ✅
- [x] CpG extraction from FASTA files
- [x] Bismark file parsing (compressed and uncompressed)
- [x] Parallel processing pipeline
- [x] HDF5 output (SummarizedExperiment compatible)
- [x] Coverage statistics calculation
- [x] QC report generation (Excel format)
- [x] Remove uncovered loci
- [x] Command-line interface with clap

### Supported Genomes
- [x] Custom FASTA files
- [x] hg19 (via UCSC download)
- [x] hg38 (via UCSC download)
- [x] mm10 (via UCSC download)
- [x] mm39 (via UCSC download)

## Version 0.2.0 (Planned)

### Enhanced Features
- [ ] Region-based filtering
- [ ] SNP masking
- [ ] Strand-specific processing
- [ ] Batch processing of multiple datasets
- [ ] Progress bar for long operations
- [ ] Better error messages

### Performance
- [ ] Optimized CpG extraction with SIMD
- [ ] Streaming Bismark file processing for very large files
- [ ] HDF5 chunked writing for memory efficiency
- [ ] Parallel HDF5 compression

### Output Formats
- [ ] Parquet output option
- [ ] BigWig export
- [ ] BEDGraph export
- [ ] Multi-sample BED format

## Version 0.3.0 (Future)

### Advanced Analysis
- [ ] Differential methylation analysis
- [ ] DMR detection
- [ ] PCA analysis
- [ ] Clustering
- [ ] Annotation integration

### Data Management
- [ ] Multi-sample combining
- [ ] Subset operations
- [ ] Data merging
- [ ] Format conversion (methrix2bsseq, etc.)

### Quality Control
- [ ] Comprehensive QC metrics
- [ ] Interactive HTML reports
- [ ] Comparative statistics
- [ ] Outlier detection

## Version 1.0.0 (Future Milestone)

### Full Feature Parity
- [ ] Complete methrix R package functionality
- [ ] All visualization functions
- [ ] All statistical operations
- [ ] Complete test coverage

### Enterprise Features
- [ ] Cloud storage support (S3, GCS)
- [ ] Distributed processing
- [ ] REST API
- [ ] Web interface

## Contributing

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Submit pull request

### Code Standards
- Follow Rust style guidelines
- Add tests for new features
- Update documentation
- Use meaningful commit messages

### Testing
```bash
# Run all tests
cargo test --all-features

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench
```

## Release Process

1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Run full test suite
4. Create git tag
5. Build release binaries
6. Create GitHub release
7. Publish crates.io (if applicable)

## Dependencies Updates

### Regular maintenance
```bash
# Check for updates
cargo outdated

# Update dependencies
cargo update

# Audit for security issues
cargo audit
```

## Breaking Changes

### Version 0.2 → 0.3
- CLI argument changes will be documented in migration guide
- H5 format will remain backward compatible
