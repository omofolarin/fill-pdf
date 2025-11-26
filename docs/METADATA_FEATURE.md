# Metadata Reporting Feature

## Overview

The CLI now reports detailed metadata about the PDF processing, including page dimensions, field counts, and any warnings/errors encountered during filling.

## Usage

```bash
fill-pdf fill \
  --template template.pdf \
  --data fields.json \
  --output output.pdf \
  --metadata metadata.json
```

## Metadata Structure

```typescript
interface ProcessingMetadata {
  pages: PageMetadata[];
  fieldsProcessed: number;
  fieldsSkipped: number;
  warnings: string[];
  errors: string[];
}

interface PageMetadata {
  pageNumber: number;
  width: number;
  height: number;
  fieldsCount: number;
}
```

## Console Output

The CLI now prints a summary to console:

```
✓ PDF filled successfully: output.pdf
  Fields processed: 7
  Fields skipped: 1
⚠️  Warnings:
    - Skipped URL image for field signature_1
    - Page 5 not found in template
```

## Tracked Events

### Warnings
- **Skipped URL images**: When image source is URL (not yet fetched)
- **Missing pages**: When field references page not in template
- **Invalid fit modes**: When unsupported fit mode specified

### Errors
- **Image decode failures**: Invalid base64 encoding
- **Image embed failures**: Unsupported image format
- **Field creation failures**: Invalid field parameters

### Page Metadata
For each page with fields:
- Page number (0-indexed)
- Width in points (1/72 inch)
- Height in points
- Number of fields on that page

## Example Output

### Successful Processing
```json
{
  "pages": [
    {
      "pageNumber": 0,
      "width": 612.0,
      "height": 792.0,
      "fieldsCount": 5
    },
    {
      "pageNumber": 1,
      "width": 612.0,
      "height": 792.0,
      "fieldsCount": 3
    }
  ],
  "fieldsProcessed": 8,
  "fieldsSkipped": 0,
  "warnings": [],
  "errors": []
}
```

### With Issues
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
  "fieldsProcessed": 6,
  "fieldsSkipped": 2,
  "warnings": [
    "Skipped URL image for field signature_1",
    "Page 2 not found in template"
  ],
  "errors": [
    "Failed to decode image photo_2: Invalid base64 encoding"
  ]
}
```

## Use Cases

1. **Debugging**: Identify which fields failed and why
2. **Validation**: Verify all fields were processed
3. **Monitoring**: Track processing success rates
4. **Template Info**: Get page dimensions for layout planning
5. **Error Reporting**: Provide detailed error context to users

## Implementation Details

### Metadata Collection
- Tracked in `PdfFieldRenderer.metadata` field
- Updated during field processing
- Returned alongside PDF bytes

### Error Handling
- Non-fatal errors logged but don't stop processing
- Fields with errors are skipped and counted
- Processing continues for remaining fields

### Console vs File
- **Console**: Summary statistics and warnings/errors
- **File**: Complete metadata with page details (JSON)

## Future Enhancements

Potential additions:
- Field-level metadata (which fields succeeded/failed)
- Processing time per page
- Image size/format information
- Font usage statistics
- Template validation warnings
