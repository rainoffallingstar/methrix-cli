#!/bin/bash
# Test workflow script for methrix-cli
# This script tests the complete pipeline with real data

set -e

# Paths
TESTDATA_DIR="/public3/home/scg9946/methrix-cli/testdata/mCall"
OUTPUT_DIR="/public3/home/scg9946/methrix-cli/testdata/mCall/rust_output"
BINARY="/public3/home/scg9946/methrix-cli/target/release/methrix"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== Methrix CLI Test Workflow ===${NC}\n"

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${YELLOW}Binary not found. Building...${NC}"
    cargo build --release
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Step 1: Check if we have reference genome
echo -e "${YELLOW}Step 1: Check reference genome${NC}"
GENOME_FASTA="$TESTDATA_DIR/hg38.fa"
CPG_RON="$TESTDATA_DIR/hg38_cpgs.ron"

if [ -f "$CPG_RON" ]; then
    echo "Found pre-extracted CpG data: $CPG_RON"
    GENOME_ARG="$CPG_RON"
elif [ -f "$GENOME_FASTA" ]; then
    echo "Found genome FASTA: $GENOME_FASTA"
    GENOME_ARG="$GENOME_FASTA"
else
    echo "No reference genome found. Please download hg38 genome:"
    echo "  wget -O $GENOME_FASTA https://hgdownload.soe.ucsc.edu/goldenPath/hg38/bigZips/hg38.fa.gz"
    echo "  gunzip $GENOME_FASTA.gz"
    echo ""
    echo "Or extract CpG sites from an existing genome:"
    echo "  $BINARY extract-cpgs --genome <genome.fa> --output $CPG_RON"
    exit 1
fi

# Step 2: Run the processing pipeline
echo ""
echo -e "${YELLOW}Step 2: Process Bismark files${NC}"
echo "Input: $TESTDATA_DIR"
echo "Output: $OUTPUT_DIR"
echo "Genome: $GENOME_ARG"
echo ""

$BINARY process \
    --input "$TESTDATA_DIR" \
    --output "$OUTPUT_DIR" \
    --genome "$GENOME_ARG" \
    --threads 8 \
    --min-coverage 1 \
    --remove-uncovered true

echo ""
echo -e "${GREEN}=== Pipeline completed successfully! ===${NC}"
echo ""
echo "Output files:"
echo "  - HDF5: $OUTPUT_DIR/methrix_data.h5"
echo "  - QC Report: $OUTPUT_DIR/CpG_coverage.xlsx"
echo ""
echo "To verify with R methrix package:"
echo "  library(methrix)"
echo "  m <- load_HDF5_methrix('$OUTPUT_DIR/methrix_data.h5')"
echo "  get_stats(m)"
