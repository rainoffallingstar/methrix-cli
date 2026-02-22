#!/bin/bash
# Build script for methrix-cli using conda environment

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== Methrix CLI Build Script ===${NC}\n"

# Check if conda is available
if ! command -v conda &> /dev/null; then
    echo -e "${RED}Error: conda not found. Please install Anaconda or Miniconda.${NC}"
    exit 1
fi

# Activate or create rust_build conda environment
if conda env list | grep -q "^rust_build "; then
    echo -e "${GREEN}Activating existing rust_build conda environment...${NC}"
    source ~/TTest/soft/MyMiniconda/etc/profile.d/conda.sh
    conda activate rust_build
else
    echo -e "${YELLOW}Creating rust_build conda environment...${NC}"
    source ~/TTest/soft/MyMiniconda/etc/profile.d/conda.sh
    conda create -n rust_build -y rust hdf5=1.12.2
    conda activate rust_build
fi

# Set HDF5_DIR to absolute path
export HDF5_DIR=$(echo $CONDA_PREFIX | sed 's|/$||')
echo "HDF5_DIR=$HDF5_DIR"

# Verify HDF5 installation
if [ ! -f "$HDF5_DIR/include/hdf5.h" ]; then
    echo -e "${RED}Error: HDF5 headers not found in $HDF5_DIR/include/${NC}"
    exit 1
fi

if [ ! -f "$HDF5_DIR/lib/libhdf5.so" ]; then
    echo -e "${RED}Error: HDF5 library not found in $HDF5_DIR/lib/${NC}"
    exit 1
fi

echo -e "${GREEN}HDF5 found in: $HDF5_DIR${NC}\n"

# Build the project
echo -e "${YELLOW}Building methrix-cli...${NC}\n"
cargo build --release

# Check if build succeeded
if [ $? -eq 0 ]; then
    echo -e "\n${GREEN}=== Build successful! ===${NC}"
    echo "Binary: target/release/methrix"
    echo ""
    echo "To use methrix-cli, run:"
    echo "  ./target/release/methrix --help"
else
    echo -e "\n${RED}=== Build failed! ===${NC}"
    exit 1
fi
