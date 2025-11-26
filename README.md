# fill-pdf

Standalone CLI tool for filling PDF forms with data - extracted from srv-ocr.

## Installation

```bash
cargo build --release
```

## Prerequisites

The CLI will automatically check for dependencies and offer to install PyPDF2 if missing.

**Required:**
- Python 3 (must be installed manually)

**Auto-installed:**
- PyPDF2 (CLI will prompt to install if missing)

**Manual installation (if needed):**
```bash
pip3 install PyPDF2
```

## Usage

### Basic Usage (Local Files)

```bash
# Default: Flattens form fields (removes interactivity)
fill-pdf fill --template template.pdf --data fields.json --output filled.pdf

# Keep interactive form fields
fill-pdf fill --template template.pdf --data fields.json --output filled.pdf --keep-fields
```

### With Metadata Output

```bash
fill-pdf fill \
  --template template.pdf \
  --data fields.json \
  --output filled.pdf \
  --metadata metadata.json
```

The metadata file contains:
- **Page dimensions** (width/height) for each page
- **Fields count** per page
- **Processing stats** (processed/skipped)
- **Warnings** (e.g., skipped URL images, missing pages)
- **Errors** (e.g., failed image decoding/embedding)

Example metadata output:
```json
{
  "pages": [
    {
      "pageNumber": 0,
      "width": 612.0,
      "height": 792.0,
      "fieldsCount": 8
    }
  ],
  "fieldsProcessed": 7,
  "fieldsSkipped": 1,
  "warnings": ["Skipped URL image for field signature_1"],
  "errors": ["Failed to decode image photo_2: Invalid base64"]
}
```

### Template from URL

**Simple GET:**
```bash
fill-pdf fill --template https://example.com/template.pdf --data fields.json --output filled.pdf
```

**With Authentication (JSON config):**
```bash
fill-pdf fill --template '{"url":"https://api.example.com/template.pdf","headers":{"Authorization":"Bearer token"}}' --data fields.json --output filled.pdf
```

**From File (JSON config):**
```bash
# Create template_config.json:
{
  "url": "https://api.example.com/templates/form.pdf",
  "method": "POST",
  "headers": {
    "Authorization": "Bearer your-token",
    "X-API-Key": "your-key"
  },
  "body": {
    "template_id": "form-123",
    "version": "latest"
  }
}

# Use it:
fill-pdf fill --template "$(cat template_config.json)" --data fields.json --output filled.pdf
```
fill-pdf fill \
  --template "https://api.example.com/templates/form.pdf" \
  --data fields.json \
  --output filled.pdf
```

### With Authentication

```bash
# Template URL can include query parameters or use authenticated endpoints
fill-pdf fill \
  --template "https://api.example.com/templates/form.pdf?token=xyz" \
  --data fields_with_url_images.json \
  --output filled.pdf
```

## JSON Data Format

### Text Field
```json
{
  "field_id": "name",
  "page": 0,
  "x": 100.0,
  "y": 200.0,
  "width": 200.0,
  "height": 20.0,
  "field_type": "text",
  "value": "John Doe",
  "font_size": 12.0,
  "alignment": "left",
  "vertical_alignment": "middle"
}
```

### Number Field
```json
{
  "field_id": "age",
  "page": 0,
  "x": 100.0,
  "y": 250.0,
  "width": 100.0,
  "height": 20.0,
  "field_type": "number",
  "value": 25
}
```

### Date Field
```json
{
  "field_id": "birthdate",
  "page": 0,
  "x": 100.0,
  "y": 300.0,
  "width": 150.0,
  "height": 20.0,
  "field_type": "date",
  "value": "01/15/1990"
}
```

### Checkbox
```json
{
  "field_id": "agree",
  "page": 0,
  "x": 100.0,
  "y": 350.0,
  "width": 15.0,
  "height": 15.0,
  "field_type": "checkbox",
  "value": true
}
```

### Radio Button
```json
{
  "field_id": "gender",
  "page": 0,
  "x": 100.0,
  "y": 400.0,
  "width": 15.0,
  "height": 15.0,
  "field_type": "radio",
  "value": "male"
}
```

### Dropdown
```json
{
  "field_id": "country",
  "page": 0,
  "x": 100.0,
  "y": 450.0,
  "width": 200.0,
  "height": 20.0,
  "field_type": "dropdown",
  "value": "USA",
  "options": ["USA", "Canada", "Mexico"]
}
```

### Signature (Base64 Image)
```json
{
  "field_id": "signature",
  "page": 0,
  "x": 100.0,
  "y": 500.0,
  "width": 150.0,
  "height": 50.0,
  "field_type": "signature",
  "value": "iVBORw0KGgoAAAANSUhEUgAAAAUA..."
}
```

### Signature from URL (Simple GET)
```json
{
  "field_id": "signature",
  "page": 0,
  "x": 100.0,
  "y": 500.0,
  "width": 150.0,
  "height": 50.0,
  "field_type": "signature",
  "value": {
    "url": "https://example.com/signatures/user123.png"
  }
}
```

### Signature from URL (With Authentication)
```json
{
  "field_id": "signature",
  "page": 0,
  "x": 100.0,
  "y": 500.0,
  "width": 150.0,
  "height": 50.0,
  "field_type": "signature",
  "value": {
    "url": "https://api.example.com/signatures/user123.png",
    "method": "POST",
    "headers": {
      "Authorization": "Bearer token",
      "Content-Type": "application/json"
    },
    "body": {
      "user_id": "123",
      "format": "png"
    }
  }
}
```

## Text Alignment Options

### Horizontal Alignment
- `left` (default)
- `center`
- `right`

### Vertical Alignment
- `top` (default)
- `middle`
- `bottom`
- `baseline`

## Features

### Template Sources
- ✅ Local PDF files
- ✅ Remote PDFs via URL (HTTP/HTTPS)
- ✅ Authenticated endpoints
- ✅ **Template caching with TTL and ETag validation**

### Caching
- ✅ Optional template caching for performance
- ✅ TTL-based expiry (default: 1 hour)
- ✅ ETag/Last-Modified validation
- ✅ Force refresh option
- ✅ Custom cache directory
- ✅ Cache management commands

See [CACHING.md](CACHING.md) for detailed documentation.

### Text Rendering
- ✅ Auto font-size reduction (90% if text doesn't fit)
- ✅ Multi-line text wrapping
- ✅ Horizontal alignment (left/center/right)
- ✅ Vertical alignment (top/middle/bottom/baseline)
- ✅ Overflow handling

### Field Types
- ✅ Text fields
- ✅ Number fields
- ✅ Date fields
- ✅ Checkboxes (interactive)
- ✅ Radio buttons (interactive)
- ✅ Dropdown menus
- ✅ Image fields (full embedding)
- ✅ Signature fields (full embedding)

### Image/Signature Sources
- ✅ Base64 encoded images (PNG, JPEG, WebP, GIF, BMP)
- ✅ Remote images via URL
- ✅ Authenticated image endpoints (headers, POST body)
- ✅ Multiple HTTP methods (GET, POST, PUT, PATCH)
- ✅ Custom headers and request bodies
- ✅ Full PDF image embedding (XObject with DCT compression)

## Complete Example

```json
[
  {
    "field_id": "full_name",
    "page": 0,
    "x": 150.0,
    "y": 100.0,
    "width": 300.0,
    "height": 25.0,
    "field_type": "text",
    "value": "John Michael Doe",
    "font_size": 14.0,
    "alignment": "left",
    "vertical_alignment": "middle"
  },
  {
    "field_id": "email",
    "page": 0,
    "x": 150.0,
    "y": 140.0,
    "width": 300.0,
    "height": 20.0,
    "field_type": "text",
    "value": "john.doe@example.com",
    "font_size": 12.0
  },
  {
    "field_id": "age",
    "page": 0,
    "x": 150.0,
    "y": 180.0,
    "width": 80.0,
    "height": 20.0,
    "field_type": "number",
    "value": 35
  },
  {
    "field_id": "terms_accepted",
    "page": 0,
    "x": 150.0,
    "y": 220.0,
    "width": 15.0,
    "height": 15.0,
    "field_type": "checkbox",
    "value": true
  }
]
```

## Coordinate System

- **Origin**: Bottom-left corner (PDF standard)
- **Units**: Points (1 point = 1/72 inch)
- **Y-axis**: Increases upward (automatically converted from top-down coordinates)

Example for A4 page (595 x 842 points):
- Top-left: (0, 0) → PDF (0, 842)
- Bottom-right: (595, 842) → PDF (595, 0)

## Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run directly
cargo run -- fill --template input.pdf --data fields.json --output output.pdf
```

## Testing

```bash
# Create test data
cat > test_fields.json << 'EOF'
[
  {
    "field_id": "name",
    "page": 0,
    "x": 100.0,
    "y": 100.0,
    "width": 200.0,
    "height": 25.0,
    "field_type": "text",
    "value": "Test User",
    "font_size": 14.0,
    "alignment": "center",
    "vertical_alignment": "middle"
  },
  {
    "field_id": "agree",
    "page": 0,
    "x": 100.0,
    "y": 150.0,
    "width": 15.0,
    "height": 15.0,
    "field_type": "checkbox",
    "value": true
  }
]
EOF

# Run
./target/release/fill-pdf fill \
  --template test.pdf \
  --data test_fields.json \
  --output output.pdf
```

## Implementation Details

Based on srv-ocr's PDF filling implementation:
- Text rendering with auto-sizing and wrapping
- Interactive form fields (checkboxes, radio buttons)
- Coordinate transformation (top-down → bottom-up)
- Multi-page support
- Font embedding (Helvetica, ZapfDingbats)

## Limitations

- Signature images are rendered as placeholders (full image embedding requires additional PDF operations)
- No custom font support (uses Helvetica and ZapfDingbats)
- No form validation
- No field dependencies or calculations
