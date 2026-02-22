#!/bin/bash
# Test script for methrix-cli with real data

set -e

# Activate conda environment with HDF5
source ~/TTest/soft/MyMiniconda/etc/profile.d/conda.sh
conda activate rust_build

export HDF5_DIR=/public3/home/scg9946/TTest/soft/MyMiniconda/envs/rust_build

# Paths
TESTDATA_DIR="/public3/home/scg9946/methrix-cli/testdata/mCall"
OUTPUT_DIR="/public3/home/scg9946/methrix-cli/testdata/mCall/rust_output"
GENOME_DIR="/public3/home/scg9946/methrix-cli/testdata/genomes"
BINARY="/public3/home/scg9946/methrix-cli/target/release/methrix"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}=== Methrix CLI Real Data Test ===${NC}\n"

# Create directories
mkdir -p "$OUTPUT_DIR"
mkdir -p "$GENOME_DIR"

# Step 1: Download reference genome
echo -e "${YELLOW}Step 1: Downloading reference genome...${NC}"
if [ ! -f "$GENOME_DIR/hg38.fa" ]; then
    echo "Downloading hg38 genome..."
    "$BINARY" download-genome --genome hg38 --output "$GENOME_DIR"
else
    echo "Using existing hg38 genome: $GENOME_DIR/hg38.fa"
fi

# Step 2: Extract CpG sites
echo -e "\n${YELLOW}Step 2: Extracting CpG sites...${NC}"
CPG_RON="$GENOME_DIR/hg38_cpgs.ron"
if [ ! -f "$CPG_RON" ]; then
    echo "Extracting CpG sites from hg38..."
    "$BINARY" extract-cp-gs --genome "$GENOME_DIR/hg38.fa" --output "$CPG_RON"
else
    echo "Using existing CpG data: $CPG_RON"
fi

# Step 3: Process a subset of samples
echo -e "\n${YELLOW}Step 3: Processing Bismark files...${NC}"
echo "Using 2 sample files for testing"

# Create a temporary directory with just 2 files
TEMP_INPUT="/tmp/methrix_test_input"
rm -rf "$TEMP_INPUT"
mkdir -p "$TEMP_INPUT"
cp "$TESTDATA_DIR"/0108ZYHHPC70311_nsort.bismark.cov.gz "$TEMP_INPUT/"
cp "$TESTDATA_DIR"/0108ZYHHPC70315_nsort.bismark.cov.gz "$TEMP_INPUT/"

"$BINARY" process \
    --input "$TEMP_INPUT" \
    --output "$OUTPUT_DIR" \
    --genome "$CPG_RON" \
    --threads 8 \
    --remove-uncovered

echo -e "\n${GREEN}=== Test completed successfully! ===${NC}"
echo "Output files:"
echo "  - HDF5: $OUTPUT_DIR/methrix_data.h5"
echo "  - QC Report: $OUTPUT_DIR/CpG_coverage.xlsx"
echo ""
echo "To verify with R methrix package:"
echo "  library(methrix)"
echo "  m <- load_HDF5_methrix('$OUTPUT_DIR/methrix_data.h5')"
echo "  get_stats(m)"
