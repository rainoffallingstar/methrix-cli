#!/usr/bin/env python3
"""
Generate test data for methx development and testing.

This script creates synthetic Bismark output files and reference genome data
for testing the methx tool.
"""

import gzip
import random
import os
from pathlib import Path


def generate_test_fasta(output_path: str, chr: str = "chr21", length: int = 100000):
    """Generate a synthetic test FASTA file with CpG sites."""
    print(f"Generating test FASTA: {output_path}")

    bases = ['A', 'C', 'G', 'T']

    with open(output_path, 'w') as f:
        f.write(f">{chr}\n")

        sequence = []
        for i in range(length):
            # Bias towards more CpG sites for testing
            if i < length - 1 and random.random() < 0.1:
                sequence.extend(['C', 'G'])
            else:
                sequence.append(random.choice(bases))

        # Write in lines of 80
        for i in range(0, len(sequence), 80):
            f.write("".join(sequence[i:i+80]) + "\n")

    print(f"  Generated {len(sequence)} bp")
    cpg_count = sum(1 for i in range(len(sequence)-1)
                     if sequence[i] == 'C' and sequence[i+1] == 'G')
    print(f"  Contains ~{cpg_count} CpG sites")


def generate_bismark_file(output_path: str, chr: str = "chr21",
                          num_cpgs: int = 1000, coverage: float = 0.8):
    """Generate a synthetic Bismark coverage file."""
    print(f"Generating Bismark file: {output_path}")

    # Generate CpG positions (1-based, Bismark format)
    cpg_positions = sorted(random.sample(range(1, 100000), num_cpgs))

    lines = []
    for pos in cpg_positions:
        if random.random() < coverage:  # 80% coverage
            meth_reads = random.randint(0, 30)
            unmeth_reads = random.randint(0, 30)

            # Bismark format: chr start end meth_reads unmeth_reads context
            lines.append(f"{chr}\t{pos}\t{pos+1}\t{meth_reads}\t{unmeth_reads}\tCG")

    # Sort by position
    lines.sort(key=lambda x: int(x.split('\t')[1]))

    # Write gzipped
    with gzip.open(output_path, 'wt') as f:
        f.write("\n".join(lines))

    print(f"  Generated {len(lines)} CpG records")


def generate_test_suite(output_dir: str):
    """Generate a complete test suite."""
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    print(f"\nGenerating test suite in: {output_dir}\n")

    # 1. Generate reference genome
    genome_dir = output_path / "genome"
    genome_dir.mkdir(exist_ok=True)

    generate_test_fasta(str(genome_dir / "test_hg19.fa"), "chr21")
    generate_test_fasta(str(genome_dir / "test_hg19.fa"), "chr22")

    # 2. Generate Bismark files
    bismark_dir = output_path / "bismark"
    bismark_dir.mkdir(exist_ok=True)

    for sample_id in range(1, 5):
        sample_name = f"sample{sample_id}"
        generate_bismark_file(
            str(bismark_dir / f"{sample_name}.bismark.cov.gz"),
            chr="chr21",
            num_cpgs=500,
            coverage=0.7 + (sample_id * 0.05)  # Varying coverage
        )

    # 3. Create README
    readme_content = """# methx Test Data

This directory contains synthetic test data for methx development and testing.

## Structure

```
test_data/
├── genome/
│   ├── test_hg19.fa    # Test reference genome (chr21, chr22)
│   └── test_hg19.fa    # Test reference genome (chr21, chr22)
└── bismark/
    ├── sample1.bismark.cov.gz
    ├── sample2.bismark.cov.gz
    ├── sample3.bismark.cov.gz
    └── sample4.bismark.cov.gz
```

## Usage

### Process test data

```bash
methx process \
  --input bismark/ \
  --output results/ \
  --genome genome/test_hg19.fa \
  --threads 4
```

### Expected results

- ~1000 CpG sites in reference genome
- 4 samples with varying coverage
- Output H5 file compatible with R methrix package

## Regenerating data

To regenerate test data:
```bash
python3 ../scripts/generate_test_data.py
```
"""

    (output_path / "README.md").write_text(readme_content)

    print(f"\n✅ Test suite generated successfully!")
    print(f"\nNext steps:")
    print(f"  1. Build methx: cargo build --release")
    print(f"  2. Run test:")
    print(f"     ./target/release/methx process \\")
    print(f"       --input {output_dir}/bismark \\")
    print(f"       --output {output_dir}/results \\")
    print(f"       --genome {output_dir}/genome/test_hg19.fa")


if __name__ == "__main__":
    import sys

    output_dir = sys.argv[1] if len(sys.argv) > 1 else "tests/data"

    if output_dir == "tests/data":
        output_dir = "methx/tests/data"

    generate_test_suite(output_dir)
