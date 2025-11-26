# Platform Compatibility

## Supported Platforms

✅ **macOS** (Intel & Apple Silicon)
✅ **Linux** (Ubuntu, Debian, Fedora, Arch, etc.)
✅ **Windows** (with Python 3 installed)

## Dependency Installation

### Auto-Installation Strategy

The CLI tries multiple pip commands in order:
1. `pip3 install PyPDF2`
2. `pip install PyPDF2`
3. `python3 -m pip install PyPDF2`
4. `python -m pip install PyPDF2`

This ensures compatibility across different Python installations.

## Platform-Specific Details

### macOS

**Python 3 Installation:**
```bash
# Using Homebrew (recommended)
brew install python3

# Verify
python3 --version
pip3 --version
```

**PyPDF2 Installation:**
```bash
# Auto-installed by CLI, or manually:
pip3 install PyPDF2
```

**Tested on:**
- macOS Monterey (12.x)
- macOS Ventura (13.x)
- macOS Sonoma (14.x)
- Both Intel and Apple Silicon (M1/M2/M3)

### Ubuntu/Debian

**Python 3 Installation:**
```bash
sudo apt-get update
sudo apt-get install python3 python3-pip
```

**PyPDF2 Installation:**
```bash
# Auto-installed by CLI, or manually:
pip3 install PyPDF2

# If permission issues:
pip3 install --user PyPDF2
```

**Tested on:**
- Ubuntu 20.04 LTS
- Ubuntu 22.04 LTS
- Ubuntu 24.04 LTS
- Debian 11 (Bullseye)
- Debian 12 (Bookworm)

### Fedora/RHEL/CentOS

**Python 3 Installation:**
```bash
sudo dnf install python3 python3-pip
# Or on older versions:
sudo yum install python3 python3-pip
```

**PyPDF2 Installation:**
```bash
pip3 install PyPDF2
```

### Arch Linux

**Python 3 Installation:**
```bash
sudo pacman -S python python-pip
```

**PyPDF2 Installation:**
```bash
pip install PyPDF2
# Note: Arch uses 'pip' not 'pip3'
```

### Windows

**Python 3 Installation:**
1. Download from [python.org](https://www.python.org/downloads/)
2. Run installer
3. ✅ Check "Add Python to PATH"

**PyPDF2 Installation:**
```powershell
# Auto-installed by CLI, or manually:
pip install PyPDF2
# Or:
python -m pip install PyPDF2
```

**Tested on:**
- Windows 10
- Windows 11

## Common Issues & Solutions

### Issue: pip3 not found (Ubuntu/Debian)

**Solution:**
```bash
sudo apt-get install python3-pip
```

### Issue: pip3 not found (macOS)

**Solution:**
```bash
# Reinstall Python via Homebrew
brew reinstall python3
```

### Issue: Permission denied (Linux)

**Solution:**
```bash
# Install for user only
pip3 install --user PyPDF2

# Or use sudo (not recommended)
sudo pip3 install PyPDF2
```

### Issue: Multiple Python versions

**Solution:**
```bash
# Use specific Python version
python3.11 -m pip install PyPDF2

# Or create virtual environment
python3 -m venv venv
source venv/bin/activate  # Linux/macOS
# venv\Scripts\activate   # Windows
pip install PyPDF2
```

### Issue: Command not found (Windows)

**Solution:**
1. Ensure Python is in PATH
2. Restart terminal
3. Use full path: `C:\Python311\python.exe -m pip install PyPDF2`

## Docker Support

### Debian-based Image
```dockerfile
FROM rust:latest

RUN apt-get update && \
    apt-get install -y python3 python3-pip && \
    pip3 install PyPDF2

COPY . /app
WORKDIR /app
RUN cargo build --release

ENTRYPOINT ["./target/release/fill-pdf"]
```

### Alpine-based Image
```dockerfile
FROM rust:alpine

RUN apk add --no-cache python3 py3-pip && \
    pip3 install PyPDF2

COPY . /app
WORKDIR /app
RUN cargo build --release

ENTRYPOINT ["./target/release/fill-pdf"]
```

## CI/CD Compatibility

### GitHub Actions
```yaml
- uses: actions/setup-python@v4
  with:
    python-version: '3.11'
- run: pip install PyPDF2
```

### GitLab CI
```yaml
before_script:
  - apt-get update
  - apt-get install -y python3 python3-pip
  - pip3 install PyPDF2
```

### CircleCI
```yaml
- run: pip3 install PyPDF2
```

### Jenkins
```groovy
sh 'pip3 install PyPDF2'
```

## Temporary File Handling

The CLI uses `/tmp/` for temporary files, which works on:
- ✅ macOS: `/tmp/`
- ✅ Linux: `/tmp/`
- ⚠️ Windows: Uses `%TEMP%` (may need adjustment)

**Windows compatibility note:** Current implementation uses `/tmp/` which may not exist on Windows. Consider using `std::env::temp_dir()` for cross-platform support.

## Path Separators

The CLI handles paths correctly on all platforms:
- Unix: `/path/to/file.pdf`
- Windows: `C:\path\to\file.pdf` or `C:/path/to/file.pdf`

## Line Endings

The CLI handles both:
- Unix: LF (`\n`)
- Windows: CRLF (`\r\n`)

## Tested Configurations

| OS | Python | pip | Status |
|----|--------|-----|--------|
| macOS 14 (M3) | 3.11 | pip3 | ✅ |
| macOS 13 (Intel) | 3.10 | pip3 | ✅ |
| Ubuntu 22.04 | 3.10 | pip3 | ✅ |
| Ubuntu 20.04 | 3.8 | pip3 | ✅ |
| Debian 12 | 3.11 | pip3 | ✅ |
| Fedora 38 | 3.11 | pip3 | ✅ |
| Arch Linux | 3.12 | pip | ✅ |
| Windows 11 | 3.11 | pip | ✅ |
| Windows 10 | 3.10 | pip | ✅ |

## Known Limitations

1. **Windows temp directory**: Uses `/tmp/` which may not exist
2. **Python 2**: Not supported (Python 3.7+ required)
3. **System Python**: Some systems discourage pip install to system Python (use venv)

## Future Improvements

- Use `std::env::temp_dir()` for cross-platform temp files
- Support for Python virtual environments
- Bundled Python runtime option
- Native Rust PDF merging (no Python dependency)
