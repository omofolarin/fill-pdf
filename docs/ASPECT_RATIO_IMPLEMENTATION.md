# Aspect Ratio Implementation

## Overview

Completed implementation of proper aspect ratio handling for images and signatures in PDF forms. The system now uses actual image dimensions to calculate scaling and positioning, supporting four different fit modes.

## Changes Made

### 1. Image Dimension Tracking

**File**: `src/renderer.rs`

Modified `image_refs` HashMap to store image dimensions:
```rust
image_refs: HashMap<String, (Ref, u32, u32)>, // (ref, width, height)
```

### 2. Fit Mode Implementation

**File**: `src/renderer.rs` - `render_embedded_image()` function

Implemented four fit modes with proper aspect ratio calculations:

#### Fill Mode
```rust
ImageFitMode::Fill => {
    (box_width, box_height, 0.0, 0.0)
}
```
- Stretches to fill box
- May distort aspect ratio

#### Contain Mode (Default)
```rust
ImageFitMode::Contain => {
    let scale = (box_width / img_width).min(box_height / img_height);
    let w = img_width * scale;
    let h = img_height * scale;
    (w, h, (box_width - w) / 2.0, (box_height - h) / 2.0)
}
```
- Fits within box
- Maintains aspect ratio
- Centers image

#### Cover Mode
```rust
ImageFitMode::Cover => {
    let scale = (box_width / img_width).max(box_height / img_height);
    let w = img_width * scale;
    let h = img_height * scale;
    (w, h, (box_width - w) / 2.0, (box_height - h) / 2.0)
}
```
- Covers entire box
- Maintains aspect ratio
- May crop image

#### Scale Down Mode
```rust
ImageFitMode::ScaleDown => {
    if img_width <= box_width && img_height <= box_height {
        (img_width, img_height, (box_width - img_width) / 2.0, (box_height - img_height) / 2.0)
    } else {
        let scale = (box_width / img_width).min(box_height / img_height);
        let w = img_width * scale;
        let h = img_height * scale;
        (w, h, (box_width - w) / 2.0, (box_height - h) / 2.0)
    }
}
```
- Only scales down if larger
- Maintains aspect ratio
- Centers image

### 3. Bug Fixes

Fixed several compilation issues:
- Corrected tuple destructuring when calling `embed_image()`
- Changed `field.id` to `field.field_id` to match struct definition
- Fixed return value order in fit mode calculations

## Testing

Created `test_aspect_ratio.json` with examples of all four fit modes using the same 10x10 pixel image in 200x100 bounding boxes.

### Test Command
```bash
./target/release/fill-pdf \
  --template template.pdf \
  --fields test_aspect_ratio.json \
  --output output.pdf
```

## Technical Details

### Scaling Calculation

For **contain** and **scale_down** modes:
```rust
let scale = (box_width / img_width).min(box_height / img_height);
```
- Takes minimum scale factor to ensure image fits within box
- Preserves aspect ratio

For **cover** mode:
```rust
let scale = (box_width / img_width).max(box_height / img_height);
```
- Takes maximum scale factor to ensure box is covered
- May crop image edges

### Centering Calculation

```rust
let offset_x = (box_width - scaled_width) / 2.0;
let offset_y = (box_height - scaled_height) / 2.0;
```
- Centers image within bounding box
- Applied to PDF transformation matrix

### PDF Transformation

```rust
content.transform([render_width, 0.0, 0.0, render_height, pdf_x + offset_x, pdf_y + offset_y]);
```
- Scales image to calculated dimensions
- Positions at calculated offset
- Maintains PDF coordinate system (bottom-up)

## Benefits

1. **Proper Aspect Ratios**: Images no longer distorted by default
2. **Flexible Positioning**: Four modes for different use cases
3. **Accurate Dimensions**: Uses actual image dimensions from decoded data
4. **Centered Images**: Automatic centering within bounding boxes
5. **Backward Compatible**: Default mode (contain) provides sensible behavior

## Future Enhancements

Potential improvements:
- Custom anchor points (top-left, bottom-right, etc.)
- Rotation support
- Clipping regions for cover mode
- Image filters (grayscale, blur, etc.)
- Multiple images per field (layers)

## Related Files

- `src/renderer.rs` - Main implementation
- `src/types.rs` - ImageFitMode enum definition
- `test_aspect_ratio.json` - Test cases
- `FEATURES.md` - Updated feature documentation
- `README.md` - Usage examples
