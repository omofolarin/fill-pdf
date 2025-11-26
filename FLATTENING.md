# PDF Form Flattening

## Overview

By default, the CLI **flattens** filled PDFs, removing interactive form fields and converting them to static content. This prevents editing and creates a cleaner final document.

## Default Behavior

```bash
# Flattens by default (removes form fields)
fill-pdf fill -t template.pdf -d fields.json -o output.pdf
```

**Result:**
- ✅ Form fields removed
- ✅ Content is static (non-editable)
- ✅ No field borders or highlights
- ✅ Smaller file size
- ✅ Professional appearance

## Keep Interactive Fields

```bash
# Preserve form fields
fill-pdf fill -t template.pdf -d fields.json -o output.pdf --keep-fields
```

**Result:**
- ✅ Form fields preserved
- ✅ Content is editable
- ✅ Field borders visible
- ✅ Larger file size
- ✅ Allows further editing

## What Gets Flattened

### Removed from PDF:
- `/AcroForm` dictionary (form structure)
- `/Annots` arrays (field annotations)
- Interactive widgets
- Field borders and highlights

### Preserved in PDF:
- Filled text content
- Checkbox/radio button marks
- Embedded images
- Page layout and structure

## Use Cases

### When to Flatten (Default)

**Invoices:**
```bash
fill-pdf fill -t invoice_template.pdf -d invoice_data.json -o invoice_final.pdf
# Customer receives non-editable invoice
```

**Certificates:**
```bash
fill-pdf fill -t certificate_template.pdf -d recipient_data.json -o certificate.pdf
# Recipient cannot modify certificate details
```

**Contracts (Final):**
```bash
fill-pdf fill -t contract_template.pdf -d contract_data.json -o signed_contract.pdf
# Executed contract is locked
```

### When to Keep Fields

**Draft Documents:**
```bash
fill-pdf fill -t template.pdf -d draft_data.json -o draft.pdf --keep-fields
# Allow further editing before finalization
```

**Templates for Others:**
```bash
fill-pdf fill -t template.pdf -d partial_data.json -o editable.pdf --keep-fields
# Recipient can fill remaining fields
```

**Multi-Step Workflows:**
```bash
# Step 1: Partial fill
fill-pdf fill -t template.pdf -d step1_data.json -o step1.pdf --keep-fields

# Step 2: Additional fill
fill-pdf fill -t step1.pdf -d step2_data.json -o step2.pdf --keep-fields

# Step 3: Final flatten
fill-pdf fill -t step2.pdf -d step3_data.json -o final.pdf
```

## Technical Details

### Flattening Process

1. **Merge overlay** - Filled content merged onto template
2. **Remove AcroForm** - Delete form structure from PDF catalog
3. **Remove annotations** - Delete field widgets from pages
4. **Preserve content** - Text and graphics remain as static content

### PyPDF2 Implementation

```python
# Remove form structure
if '/AcroForm' in template.trailer['/Root']:
    del template.trailer['/Root']['/AcroForm']

# Remove field annotations
for page in template.pages:
    if '/Annots' in page:
        del page['/Annots']
```

## File Size Impact

### Example: Invoice Template

**With fields (--keep-fields):**
- Size: 125 KB
- Contains: Form structure + annotations + content

**Flattened (default):**
- Size: 98 KB
- Contains: Content only
- **Reduction: ~22%**

### Example: Multi-Page Form

**With fields:**
- Size: 450 KB
- 10 pages with 50 fields

**Flattened:**
- Size: 320 KB
- **Reduction: ~29%**

## Visual Differences

### With Fields (--keep-fields)
```
┌─────────────────────┐
│ Name: [John Doe   ]│ ← Field border visible
│ Email: [john@...  ]│ ← Editable text field
│ ☑ Agree to terms   │ ← Interactive checkbox
└─────────────────────┘
```

### Flattened (default)
```
┌─────────────────────┐
│ Name: John Doe      │ ← Static text
│ Email: john@...     │ ← Static text
│ ☑ Agree to terms    │ ← Static checkmark
└─────────────────────┘
```

## Compatibility

### PDF Readers
- ✅ Adobe Acrobat Reader
- ✅ Preview (macOS)
- ✅ Chrome PDF Viewer
- ✅ Firefox PDF Viewer
- ✅ Edge PDF Viewer

### Printing
- ✅ Flattened PDFs print identically to originals
- ✅ No loss of visual fidelity
- ✅ Faster rendering (no form processing)

### Archival
- ✅ Flattened PDFs are better for long-term storage
- ✅ No dependency on form field support
- ✅ Consistent rendering across all viewers

## Best Practices

1. **Default to flattening** - Use for final documents
2. **Keep fields for drafts** - Use `--keep-fields` during development
3. **Test before production** - Verify flattened output looks correct
4. **Archive flattened versions** - Store final documents as flattened
5. **Document the choice** - Note in workflows when fields are preserved

## Troubleshooting

### Fields still visible after flattening
```bash
# Ensure you're not using --keep-fields
fill-pdf fill -t template.pdf -d fields.json -o output.pdf
# (no --keep-fields flag)
```

### Content missing after flattening
```bash
# Check if template has proper content layers
# Some templates may have fields without background content
# Solution: Use --keep-fields or fix template
```

### File size increased after flattening
```bash
# Rare, but can happen with complex templates
# Usually indicates embedded fonts or images in form structure
# Flattening is still recommended for security
```

## Security Considerations

### Flattened PDFs (Default)
- ✅ Cannot be edited without PDF editor
- ✅ Prevents casual tampering
- ✅ Suitable for official documents
- ⚠️ Not cryptographically signed (use digital signatures for that)

### PDFs with Fields (--keep-fields)
- ⚠️ Can be edited by anyone
- ⚠️ Not suitable for final documents
- ⚠️ Field values can be changed
- ✅ Useful for collaborative workflows

## Examples

### Invoice Generation
```bash
# Generate non-editable invoice
fill-pdf fill \
  -t invoice_template.pdf \
  -d invoice_123.json \
  -o invoices/invoice_123.pdf
# Default: flattened
```

### Certificate Generation
```bash
# Generate tamper-resistant certificate
fill-pdf fill \
  -t certificate_template.pdf \
  -d recipient_data.json \
  -o certificates/john_doe.pdf
# Default: flattened
```

### Draft Contract
```bash
# Generate editable draft for review
fill-pdf fill \
  -t contract_template.pdf \
  -d draft_terms.json \
  -o drafts/contract_draft.pdf \
  --keep-fields
# Keeps fields for editing
```

### Batch Processing
```bash
# Flatten all generated documents
for file in data/*.json; do
  name=$(basename "$file" .json)
  fill-pdf fill -t template.pdf -d "$file" -o "output/${name}.pdf"
done
# All outputs are flattened by default
```

## Summary

- **Default: Flatten** - Removes form fields, creates static PDF
- **Optional: Keep fields** - Use `--keep-fields` flag
- **Recommendation**: Flatten for final documents, keep fields for drafts
- **File size**: Flattening typically reduces size by 20-30%
- **Security**: Flattened PDFs are harder to tamper with
