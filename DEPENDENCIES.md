# Dependency Management

## Overview

The CLI automatically checks for required dependencies and offers to install missing packages.

## Required Dependencies

### Python 3 (Manual Installation Required)

**Check if installed:**
```bash
python3 --version
```

**Installation:**

**macOS:**
```bash
brew install python3
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install python3 python3-pip
```

**Windows:**
Download from [python.org](https://www.python.org/downloads/)

### PyPDF2 (Auto-Installed)

The CLI will automatically detect if PyPDF2 is missing and offer to install it.

## First Run Experience

### Scenario 1: All Dependencies Present
```bash
$ fill-pdf fill -t template.pdf -d fields.json -o output.pdf
📥 Fetching template from URL...
✓ PDF filled successfully: output.pdf
```

### Scenario 2: PyPDF2 Missing
```bash
$ fill-pdf fill -t template.pdf -d fields.json -o output.pdf
⚠️  PyPDF2 is not installed.
Would you like to install it now? (y/N): y
📦 Installing PyPDF2...
✓ PyPDF2 installed successfully
📥 Fetching template from URL...
✓ PDF filled successfully: output.pdf
```

### Scenario 3: Python 3 Missing
```bash
$ fill-pdf fill -t template.pdf -d fields.json -o output.pdf
Error: Python 3 is not installed. Please install Python 3 first.
```

## Manual Installation

If auto-installation fails or you prefer manual installation:

```bash
# Install PyPDF2
pip3 install PyPDF2

# Verify installation
python3 -c "import PyPDF2; print('PyPDF2 installed')"
```

## Troubleshooting

### pip3 not found
```bash
# macOS
brew install python3

# Ubuntu/Debian
sudo apt-get install python3-pip

# Verify
pip3 --version
```

### Permission denied during installation
```bash
# Use --user flag
pip3 install --user PyPDF2

# Or use sudo (not recommended)
sudo pip3 install PyPDF2
```

### PyPDF2 installation fails
```bash
# Update pip first
pip3 install --upgrade pip

# Try again
pip3 install PyPDF2

# Or specify version
pip3 install PyPDF2==3.0.1
```

### Multiple Python versions
```bash
# Check which python3 is being used
which python3

# Use specific version
python3.11 -m pip install PyPDF2
```

## CI/CD Integration

### GitHub Actions
```yaml
- name: Setup Python
  uses: actions/setup-python@v4
  with:
    python-version: '3.11'

- name: Install dependencies
  run: pip3 install PyPDF2

- name: Generate PDFs
  run: fill-pdf fill -t template.pdf -d fields.json -o output.pdf
```

### Docker
```dockerfile
FROM rust:latest

# Install Python and PyPDF2
RUN apt-get update && \
    apt-get install -y python3 python3-pip && \
    pip3 install PyPDF2

# Copy and build fill-pdf
COPY . /app
WORKDIR /app
RUN cargo build --release

ENTRYPOINT ["./target/release/fill-pdf"]
```

### GitLab CI
```yaml
before_script:
  - apt-get update
  - apt-get install -y python3 python3-pip
  - pip3 install PyPDF2

generate_pdfs:
  script:
    - fill-pdf fill -t template.pdf -d fields.json -o output.pdf
```

## Dependency Versions

### Tested Versions
- Python: 3.8, 3.9, 3.10, 3.11, 3.12
- PyPDF2: 3.0.0+

### Minimum Requirements
- Python: 3.7+
- PyPDF2: 2.0.0+

## Why PyPDF2?

**Advantages:**
- ✅ Pure Python (no system dependencies)
- ✅ Widely used and well-tested
- ✅ Handles complex PDF operations
- ✅ Active maintenance
- ✅ Easy installation

**Alternatives considered:**
- pypdf (PyPDF2 fork) - Compatible
- pdfrw - Less feature-complete
- pikepdf - Requires C++ dependencies

## Security Considerations

- PyPDF2 is installed from PyPI (official Python package index)
- Auto-installation requires user confirmation
- No automatic execution of untrusted code
- Dependencies are pinned to stable versions

## Offline Usage

For environments without internet access:

```bash
# Download PyPDF2 wheel
pip3 download PyPDF2 -d ./wheels

# Transfer wheels to offline machine

# Install from local wheels
pip3 install --no-index --find-links=./wheels PyPDF2
```

## Future Improvements

Potential enhancements:
- Support for alternative PDF libraries
- Bundled Python runtime option
- Native Rust PDF merging (no Python dependency)
- Docker image with all dependencies pre-installed
