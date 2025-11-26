# PDF to Image Conversion

## Overview

The `to-image` command converts PDF pages to high-quality PNG or JPEG images, useful for:
- Visual verification of filled forms
- AI-based text extraction (Gemini, GPT-4 Vision, Claude)
- OCR processing
- Thumbnail generation
- Print preview

## Installation Requirements

### Python Dependencies
```bash
pip install pdf2image
```

### System Dependencies

**macOS:**
```bash
brew install poppler
```

**Ubuntu/Debian:**
```bash
apt-get install poppler-utils
```

**Windows:**
Download and install from: http://blog.alivate.com.au/poppler-windows/

## Usage

### Basic Usage

```bash
# Single PDF
fill-pdf to-image filled.pdf --output-dir ./images

# Multiple PDFs
fill-pdf to-image file1.pdf file2.pdf file3.pdf --output-dir ./images
```

### Format Options

```bash
# PNG (default, lossless)
fill-pdf to-image filled.pdf --output-dir ./images --format png

# JPEG (smaller file size)
fill-pdf to-image filled.pdf --output-dir ./images --format jpeg
```

### DPI Settings

```bash
# Standard quality (300 DPI - recommended for OCR/AI)
fill-pdf to-image filled.pdf --output-dir ./images --dpi 300

# High quality (600 DPI - for printing)
fill-pdf to-image filled.pdf --output-dir ./images --dpi 600

# Fast preview (150 DPI)
fill-pdf to-image filled.pdf --output-dir ./images --dpi 150
```

## Output Format

Files are named: `{pdf-name}_{page-number}.{format}`

**Example:**
```
test-images/
├── filled_001.png
├── filled_002.png
└── filled_003.png
```

## DPI Recommendations

| Use Case | DPI | File Size | Quality |
|----------|-----|-----------|---------|
| Preview | 150 | Small | Good |
| OCR/AI | 300 | Medium | Excellent |
| Print | 600 | Large | Perfect |

## Integration with AI Services

### Gemini Example

```python
import google.generativeai as genai
from PIL import Image

# Convert PDF to images
# fill-pdf to-image form.pdf --output-dir ./images

# Load image
img = Image.open('images/form_001.png')

# Extract text with Gemini
model = genai.GenerativeModel('gemini-pro-vision')
response = model.generate_content([
    "Extract all text from this form",
    img
])

print(response.text)
```

### GPT-4 Vision Example

```python
import openai
import base64

# Convert PDF to images
# fill-pdf to-image form.pdf --output-dir ./images

# Encode image
with open('images/form_001.png', 'rb') as f:
    image_data = base64.b64encode(f.read()).decode()

# Extract text with GPT-4 Vision
response = openai.ChatCompletion.create(
    model="gpt-4-vision-preview",
    messages=[{
        "role": "user",
        "content": [
            {"type": "text", "text": "Extract all text from this form"},
            {"type": "image_url", "image_url": f"data:image/png;base64,{image_data}"}
        ]
    }]
)

print(response.choices[0].message.content)
```

## Batch Processing

```bash
# Convert all PDFs in a directory
for pdf in *.pdf; do
    fill-pdf to-image "$pdf" --output-dir ./images
done

# Or use shell expansion
fill-pdf to-image *.pdf --output-dir ./images
```

## Performance

**Conversion speed** (approximate):
- 150 DPI: ~0.5s per page
- 300 DPI: ~1s per page
- 600 DPI: ~2s per page

**File sizes** (approximate per page):
- PNG 150 DPI: ~200KB
- PNG 300 DPI: ~400KB
- PNG 600 DPI: ~1.5MB
- JPEG 300 DPI: ~150KB

## Troubleshooting

### "No module named 'pdf2image'"
```bash
pip install pdf2image
```

### "Unable to get page count. Is poppler installed?"
```bash
# macOS
brew install poppler

# Ubuntu
apt-get install poppler-utils
```

### "Permission denied"
Ensure output directory is writable:
```bash
mkdir -p ./images
chmod 755 ./images
```

## Why Use This Over Screenshot Tools?

1. **Consistent Quality**: Always 300 DPI, not screen resolution
2. **Batch Processing**: Convert multiple PDFs at once
3. **Automation**: Scriptable for CI/CD pipelines
4. **Accuracy**: Pixel-perfect rendering from PDF data
5. **AI-Ready**: Optimized DPI for OCR and vision models
