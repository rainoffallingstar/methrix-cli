# methx Build Instructions

## Building from Source

### Prerequisites

1. **Rust toolchain** (1.75+):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **HDF5 libraries**:

   **Ubuntu/Debian**:
   ```bash
   sudo apt-get update
   sudo apt-get install -y \
     libhdf5-dev \
     libhdf5-serial-dev \
     hdf5-tools
   ```

   **macOS**:
   ```bash
   brew install hdf5
   ```

   **Windows**:
   - Download from https://www.hdfgroup.org/downloads/index.html
   - Install and add to PATH
   - Set `HDF5_DIR` environment variable

### Build Steps

```bash
# Clone the repository (if in methrix root)
cd methx

# Build release version (optimized)
cargo build --release

# The binary will be at:
#   Linux/macOS: target/release/methx
#   Windows: target/release/methx.exe

# Optional: Install to system path
sudo cp target/release/methx /usr/local/bin/
```

### Development Build

```bash
# Build with debug symbols
cargo build

# Run tests
cargo test

# Run specific test
cargo test test_cpg_extraction

# Run with debug logging
RUST_LOG=debug cargo run -- process --help
```

### Cross-compilation

#### Linux to Windows

```bash
cargo install cross
cross build --target x86_64-pc-windows-gnu --release
```

#### Linux to macOS

```bash
cross build --target x86_64-apple-darwin --release
```

## Running the Binary

```bash
# Show help
./target/release/methx --help

# Process Bismark files
./target/release/methx process \
  --input bismark_output/ \
  --output results/ \
  --genome hg19.fa \
  --threads 8

# Extract CpGs
./target/release/methx extract-cpgs \
  --genome hg19.fa \
  --output hg19_cpgs.ron

# Generate QC report
./target/release/methx qc-report \
  --input results/ \
  --output qc.xlsx
```

## Docker Build

### Dockerfile

```dockerfile
FROM rust:1.75-slim as builder

# Install HDF5
RUN apt-get update && \
    apt-get install -y \
        libhdf5-dev \
        pkg-config && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y \
        libhdf5-serial-dev \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/methx /usr/local/bin/

ENTRYPOINT ["methx"]
```

### Build and run

```bash
# Build
docker build -t methx .

# Run
docker run -v $(pwd)/data:/data methx \
  process --input /data/bismark --output /data/results --genome /data/hg19.fa
```

## Installation Packages

### Debian Package

```bash
# Install cargo-deb
cargo install cargo-deb

# Build package
cargo deb --no-build

# Install
sudo dpkg -i target/debian/methx*.deb
```

### RPM Package

```bash
# Install cargo-generate-rpm
cargo install cargo-generate-rpm

# Build package
cargo generate-rpm

# Install
sudo rpm -i target/generate-rpm/methx*.rpm
```

## Verification

### Test installation

```bash
# Check version
methx --version

# Run help
methx --help

# Test basic functionality
methx extract-cpgs --help
methx process --help
methx qc-report --help
```

### Test with sample data

```bash
# (If sample data is available)
methx process \
  --input tests/data/bismark/ \
  --output /tmp/test_output/ \
  --genome tests/data/hg19.fa \
  --threads 2
```

## Troubleshooting Build Issues

### "Cannot find -lhdf5"

**Solution**: Install HDF5 development libraries (see Prerequisites above).

### "Linking error on Windows"

**Solution**: 
1. Ensure HDF5 is installed
2. Set `HDF5_DIR` environment variable
3. Add `%HDF5_DIR%\bin` to PATH

### "needletail compilation error"

**Solution**: Update Rust and dependencies:
```bash
cargo update
cargo clean
cargo build
```

### macOS: "Library not loaded: @rpath/libhdf5.200.dylib"

**Solution**:
```bash
brew reinstall hdf5
export HDF5_DIR=$(brew --prefix hdf5)
cargo clean
cargo build
```

## Advanced Build Options

### Static linking

```bash
# Build static binary (for systems without HDF5)
cargo build --release --features static
```

### Custom features

```bash
# Build with download feature
cargo build --release --features download

# Build with all features
cargo build --release --all-features
```

### Optimized for your CPU

```bash
# Native CPU optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Continuous Integration

### GitHub Actions

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install HDF5
        run: sudo apt-get install -y libhdf5-dev
      - name: Build
        run: cargo build --release --all-features
      - name: Test
        run: cargo test --all-features
      - name: Upload binary
        uses: actions/upload-artifact@v3
        with:
          name: methx
          path: target/release/methx
```
