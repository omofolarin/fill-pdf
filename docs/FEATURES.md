# Complete Feature List

## ✅ Implemented Features

### Template Loading
- [x] Local PDF files
- [x] Remote PDFs via URL (HTTP/HTTPS)
- [x] **Authenticated template endpoints**
  - [x] Custom headers (Authorization, API keys, etc.)
  - [x] POST/PUT/PATCH methods
  - [x] JSON request bodies
- [x] Flexible template source (path, simple URL, or JSON config)

### Text Fields
- [x] Basic text rendering
- [x] Number formatting
- [x] Date formatting
- [x] Dropdown values
- [x] Font size control (default: 12pt, min: 6pt)
- [x] Auto font-size reduction (90% if doesn't fit)
- [x] Multi-line text wrapping
- [x] Horizontal alignment (left/center/right)
- [x] Vertical alignment (top/middle/bottom/baseline)
- [x] Overflow handling (renders full text if can't fit)

### Interactive Fields
- [x] Checkboxes (ZapfDingbats checkmark)
- [x] Radio buttons (ZapfDingbats filled circle)
- [x] Dropdown menus (with options list)
- [x] Form field state preservation

### Images/Signatures
- [x] Base64 encoded images
- [x] Remote images via URL (GET)
- [x] Authenticated image endpoints
  - [x] Custom headers (Authorization, API keys, etc.)
  - [x] POST/PUT/PATCH methods
  - [x] JSON request bodies
- [x] Multiple image formats (PNG, JPEG, WebP, GIF, BMP)
- [x] Automatic image fetching before PDF generation
- [x] Graceful error handling (skip failed images)
- [x] **Full PDF image embedding** (XObject with DCT compression)
- [x] Image deduplication (same field_id reuses XObject)

### PDF Operations
- [x] Multi-page support
- [x] Coordinate transformation (top-down → bottom-up)
- [x] Template + overlay merging (PyPDF2)
- [x] Font embedding (Helvetica, ZapfDingbats)
- [x] **Form flattening (default)** - Removes interactive fields
- [x] Optional field preservation (--keep-fields)

### Developer Experience
- [x] CLI interface with clap
- [x] JSON configuration
- [x] Async operations (tokio)
- [x] Clear error messages
- [x] Progress indicators
- [x] Comprehensive examples

## ⚠️ Partial Implementation

None - all features fully implemented!

## ❌ Not Implemented

### Advanced PDF Features
- [ ] Custom font embedding (TrueType/OpenType)
- [ ] Non-Latin character support
- [ ] Form validation rules
- [ ] Calculated fields
- [ ] Conditional field visibility
- [ ] Field dependencies

### Advanced Image Features
- [ ] Image compression/optimization
- [ ] Image format conversion
- [ ] Image resizing/scaling
- [ ] Image rotation
- [ ] Transparency handling

### Security Features
- [ ] PDF encryption
- [ ] Digital signatures
- [ ] Permission controls
- [ ] Watermarking

### Performance Features
- [ ] Parallel image fetching
- [ ] Image caching
- [ ] Template caching
- [ ] Batch processing

## Comparison with srv-ocr

| Feature | srv-ocr | fill-pdf | Notes |
|---------|---------|----------|-------|
| **Core Functionality** |
| Text fields | ✅ | ✅ | Full parity |
| Checkboxes | ✅ | ✅ | Full parity |
| Radio buttons | ✅ | ✅ | Full parity |
| Dropdowns | ✅ | ✅ | Full parity |
| Multi-line text | ✅ | ✅ | Full parity |
| Font auto-sizing | ✅ | ✅ | Full parity |
| Text alignment | ✅ | ✅ | Full parity |
| **Image Handling** |
| Base64 images | ✅ | ✅ | Full parity |
| URL images | ❌ | ✅ | **New feature** |
| Authenticated URLs | ❌ | ✅ | **New feature** |
| POST/PUT requests | ❌ | ✅ | **New feature** |
| Full image embedding | ✅ | ✅ | **Full parity** |
| Image field type | ✅ | ✅ | **Full parity** |
| **Template Loading** |
| Local files | ✅ | ✅ | Full parity |
| Simple URL templates | ❌ | ✅ | **New feature** |
| Authenticated URLs | ❌ | ✅ | **New feature** |
| POST/PUT requests | ❌ | ✅ | **New feature** |
| **Infrastructure** |
| Database | ✅ SurrealDB | ❌ | Removed |
| Web server | ✅ Actix | ❌ | Removed |
| Authentication | ✅ JWT | ❌ | Removed |
| Cloud storage | ✅ AWS S3 | ❌ | Removed |
| Async operations | ✅ | ✅ | Full parity |

## Usage Examples

### 1. Simple Local Fill
```bash
fill-pdf fill \
  --template form.pdf \
  --data fields.json \
  --output filled.pdf
```

### 2. Remote Template
```bash
fill-pdf fill \
  --template "https://api.example.com/templates/form.pdf" \
  --data fields.json \
  --output filled.pdf
```

### 3. With URL Images
```json
{
  "field_id": "signature",
  "field_type": "signature",
  "value": {
    "url": "https://api.example.com/signatures/123.png",
    "headers": {
      "Authorization": "Bearer token"
    }
  }
}
```

### 4. Dynamic Signature Generation
```json
{
  "field_id": "signature",
  "field_type": "signature",
  "value": {
    "url": "https://signature-api.com/generate",
    "method": "POST",
    "headers": {
      "Authorization": "Bearer token",
      "Content-Type": "application/json"
    },
    "body": {
      "text": "John Doe",
      "style": "cursive",
      "format": "png"
    }
  }
}
```

## Next Steps

All core features from srv-ocr are now implemented! Optional enhancements:


## Image Aspect Ratio Handling (NEW)

### Fit Modes

The tool now properly handles aspect ratios using actual image dimensions:

#### 1. Fill Mode (`fit_mode: "fill"`)
- Stretches image to fill entire bounding box
- May distort aspect ratio
- No centering offset
- Use when exact box coverage is required

#### 2. Contain Mode (`fit_mode: "contain"`, **default**)
- Scales image to fit within bounding box
- Maintains aspect ratio
- Centers image within box
- Leaves empty space if aspect ratios don't match
- Best for preserving image quality

#### 3. Cover Mode (`fit_mode: "cover"`)
- Scales image to cover entire bounding box
- Maintains aspect ratio
- May crop parts of image
- Centers image within box
- Best for background images

#### 4. Scale Down Mode (`fit_mode: "scale_down"`)
- Only scales down if image is larger than box
- Maintains aspect ratio
- Centers image within box
- Leaves image at original size if smaller than box
- Best for thumbnails

### Technical Implementation

- Image dimensions tracked in `image_refs` HashMap: `(Ref, width, height)`
- Scaling calculations use actual image dimensions vs. bounding box dimensions
- Centering offsets calculated based on scaled dimensions
- PDF transformation matrix applies both scaling and positioning

### Usage Examples

#### Basic Image with Aspect Ratio
```json
{
  "field_id": "photo",
  "page": 0,
  "x": 100,
  "y": 100,
  "width": 200,
  "height": 150,
  "image": "base64_encoded_image_data",
  "fit_mode": "contain"
}
```

#### Signature with Cover Mode
```json
{
  "field_id": "signature",
  "page": 0,
  "x": 50,
  "y": 500,
  "width": 300,
  "height": 100,
  "signature": "base64_encoded_signature_data",
  "fit_mode": "cover"
}
```

#### Test All Fit Modes
See `test_aspect_ratio.json` for examples of all four fit modes with the same image in different bounding boxes.

```bash
./target/release/fill-pdf \
  --template template.pdf \
  --fields test_aspect_ratio.json \
  --output output.pdf
```
