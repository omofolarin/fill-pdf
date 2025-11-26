# fill-pdf Implementation Guide

## What Was Extracted from srv-ocr

### Core Components

1. **PDF Rendering Logic** (`src/renderer.rs`)
   - Source: `srv-ocr/src/coordinates/renderer.rs`
   - Extracted: `PdfFieldRenderer` struct and text rendering methods
   - Simplified: Removed checkbox/radio/signature handling (only text fields)

2. **Type Definitions** (`src/types.rs`)
   - Source: `srv-ocr/src/pdf_fields/models.rs` + `srv-ocr/src/coordinates/types.rs`
   - Extracted: `PdfDocument`, `PdfPageInfo`, `FieldData`, `FieldType`
   - Simplified: Removed database-specific fields (Thing IDs, timestamps)

3. **PDF Merging** (`src/merge.rs`)
   - Source: `srv-ocr/src/coordinates/merge.rs`
   - Extracted: PyPDF2-based template + overlay merging
   - Simplified: Removed async/web dependencies

### Dependencies Used

From srv-ocr's Cargo.toml:
- `lopdf = "0.36.0"` - PDF parsing
- `pdf-writer = "0.11.0"` - PDF generation
- `image = "0.25.6"` - Image handling (for future signature support)
- `base64 = "0.22.1"` - Base64 encoding (for future image embedding)

Added for CLI:
- `clap = "4.0"` - Command-line argument parsing
- `anyhow = "1.0"` - Error handling

### What Was NOT Included

1. **Database Layer**
   - No SurrealDB integration
   - No database models or queries
   - All data comes from JSON files

2. **Web Server**
   - No Actix-web
   - No HTTP routes or handlers
   - Pure CLI tool

3. **External Services**
   - No AWS S3 integration
   - No authentication
   - No audit logging

4. **Advanced Image Features**
   - Signature images rendered as placeholders (full embedding requires complex PDF operations)
   - No image compression or optimization

## How to Use

### 1. Build

```bash
cd /Users/omofolarin/Documents/clients-work/fill-pdf
cargo build --release
```

### 2. Prepare Data

Create a JSON file with field data:

```json
[
  {
    "field_id": "name",
    "page": 0,
    "x": 100.0,
    "y": 200.0,
    "width": 200.0,
    "height": 20.0,
    "value": "John Doe",
    "field_type": "text"
  }
]
```

### 3. Fill PDF

```bash
./target/release/fill-pdf fill \
  --template input.pdf \
  --data fields.json \
  --output filled.pdf
```

## Coordinate System

- **Origin**: Bottom-left corner (PDF standard)
- **Units**: Points (1 point = 1/72 inch)
- **Y-axis**: Increases upward

Example for A4 page (595 x 842 points):
- Top-left: (0, 842)
- Bottom-right: (595, 0)

## Future Enhancements

To add remaining features from srv-ocr:

1. **Full Image Embedding**
   - Copy image stream creation from `srv-ocr/src/coordinates/renderer.rs`
   - Add JPEG/PNG compression
   - Implement proper XObject references

2. **Custom Fonts**
   - Add TrueType font embedding
   - Support for non-Latin characters

3. **Form Validation**
   - Add field validation rules
   - Implement error reporting

4. **Field Dependencies**
   - Add calculated fields
   - Implement conditional visibility

## Differences from srv-ocr

| Feature | srv-ocr | fill-pdf |
|---------|---------|----------|
| Database | SurrealDB | JSON files |
| Server | Actix-web | CLI only |
| Auth | JWT + Firebase | None |
| Storage | AWS S3 | Local files |
| Text Fields | ✅ Full support | ✅ Full support |
| Checkboxes | ✅ Interactive | ✅ Interactive |
| Radio Buttons | ✅ Interactive | ✅ Interactive |
| Dropdowns | ✅ Interactive | ✅ Interactive |
| Signatures | ✅ Full embedding | ⚠️ Placeholder |
| Multi-line Text | ✅ | ✅ |
| Font Auto-sizing | ✅ | ✅ |
| Text Alignment | ✅ | ✅ |
| Async | Yes | No |
| Dependencies | 50+ crates | 8 crates |

## Maintenance

To sync with srv-ocr updates:

1. Check `srv-ocr/src/coordinates/renderer.rs` for rendering improvements
2. Check `srv-ocr/src/pdf_fills/service.rs` for text handling updates
3. Copy relevant logic (remove database/web dependencies)
4. Test with sample PDFs

## Testing

```bash
# Create test data
cat > test_fields.json << EOF
[
  {
    "field_id": "test",
    "page": 0,
    "x": 100.0,
    "y": 700.0,
    "width": 200.0,
    "height": 20.0,
    "value": "Test Value",
    "field_type": "text"
  }
]
EOF

# Run
./target/release/fill-pdf fill \
  --template test.pdf \
  --data test_fields.json \
  --output output.pdf
```
