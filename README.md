[![Tests](https://github.com/brentp/bedder-py-wrappers/actions/workflows/test.yml/badge.svg)](https://github.com/brentp/bedder-py-wrappers/actions/workflows/test.yml)

# BCF Reader

A BCF/VCF reader with Python bindings using Rust and HTSlib for use in bedder

## Prerequisites

- Rust (install via [rustup](https://rustup.rs/))
- Python 3.8+
- uv (install via `curl -LsSf https://astral.sh/uv/install.sh | sh`)

## Setup

```bash
# 1. Clone the repository:
git clone <repository-url>
cd bcf-reader

# 2. Create a Python environment with uv:
uv venv bedder-py-env

# 3. Install development dependencies:
source bedder-py-env/bin/activate
uv pip install pytest maturin

# 4. Build and install the package in development mode:
maturin develop --uv

# 5. Run Tests

pytest tests/ -v
```

## Usage

```python
from bcf_reader import PyReader

# Open a VCF/BCF file
reader = PyReader("path/to/file.vcf.gz")

# Iterate through records
for record in reader:
    # Access basic fields
    print(record.chrom, record.pos, record.ref_allele, record.alt_alleles)
    
    # Access INFO fields
    if record.has_info("AF"):
        af = record.info("AF")  # Returns list of values
```

## Building for Distribution

To build a wheel:
```bash
maturin build
``` 
