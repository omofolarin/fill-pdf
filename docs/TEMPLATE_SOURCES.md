# Template Source Options

The CLI accepts templates from multiple sources with flexible configuration.

## Source Types

### 1. Local File Path

**Usage:**
```bash
fill-pdf fill --template /path/to/template.pdf --data fields.json --output output.pdf
```

**When to use:**
- Template stored locally
- No network access needed
- Fastest option

### 2. Simple URL (GET)

**Usage:**
```bash
fill-pdf fill --template https://example.com/template.pdf --data fields.json --output output.pdf
```

**When to use:**
- Public template URL
- No authentication required
- Simple HTTP GET

### 3. URL with Configuration (JSON)

**Inline JSON:**
```bash
fill-pdf fill \
  --template '{"url":"https://api.example.com/template.pdf","headers":{"Authorization":"Bearer token"}}' \
  --data fields.json \
  --output output.pdf
```

**From File:**
```bash
# Create template_config.json
fill-pdf fill --template "$(cat template_config.json)" --data fields.json --output output.pdf
```

**When to use:**
- Authenticated endpoints
- Custom HTTP methods (POST/PUT/PATCH)
- Request body required
- Custom headers needed

## URL Configuration Format

```json
{
  "url": "https://api.example.com/templates/form.pdf",
  "method": "POST",
  "headers": {
    "Authorization": "Bearer your-token-here",
    "X-API-Key": "your-api-key",
    "Content-Type": "application/json"
  },
  "body": {
    "template_id": "form-123",
    "version": "latest",
    "format": "pdf"
  }
}
```

### Fields

- **url** (required): Template endpoint URL
- **method** (optional): HTTP method - GET (default), POST, PUT, PATCH
- **headers** (optional): Custom HTTP headers (auth, API keys, etc.)
- **body** (optional): JSON request body

## Examples

### Example 1: Bearer Token Authentication

```json
{
  "url": "https://api.example.com/template.pdf",
  "headers": {
    "Authorization": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

### Example 2: API Key Authentication

```json
{
  "url": "https://api.example.com/template.pdf",
  "headers": {
    "X-API-Key": "sk_live_abc123xyz789"
  }
}
```

### Example 3: POST with Request Body

```json
{
  "url": "https://api.example.com/templates/generate",
  "method": "POST",
  "headers": {
    "Authorization": "Bearer token",
    "Content-Type": "application/json"
  },
  "body": {
    "template_id": "invoice-v2",
    "locale": "en-US",
    "format": "pdf"
  }
}
```

### Example 4: Custom Headers

```json
{
  "url": "https://cdn.example.com/templates/form.pdf",
  "headers": {
    "X-Tenant-ID": "tenant-123",
    "X-Request-ID": "req-456",
    "User-Agent": "FillPDF/1.0"
  }
}
```

## Comparison with Image Sources

Templates and images now have **identical** URL configuration:

| Feature | Templates | Images |
|---------|-----------|--------|
| Local files | ✅ Path | ✅ Base64 |
| Simple URL | ✅ GET | ✅ GET |
| Custom headers | ✅ JSON config | ✅ JSON config |
| HTTP methods | ✅ POST/PUT/PATCH | ✅ POST/PUT/PATCH |
| Request body | ✅ JSON | ✅ JSON |

## Use Cases

### Use Case 1: Template Service with Auth
```bash
# Template behind authentication
fill-pdf fill \
  --template '{"url":"https://templates.service.com/api/v1/forms/invoice","headers":{"Authorization":"Bearer $TOKEN"}}' \
  --data invoice_data.json \
  --output invoice.pdf
```

### Use Case 2: Dynamic Template Selection
```bash
# POST to select template variant
fill-pdf fill \
  --template '{"url":"https://api.example.com/templates","method":"POST","body":{"type":"contract","language":"en"}}' \
  --data contract_data.json \
  --output contract.pdf
```

### Use Case 3: Multi-Tenant System
```bash
# Tenant-specific template
fill-pdf fill \
  --template '{"url":"https://api.example.com/template.pdf","headers":{"X-Tenant-ID":"tenant-123"}}' \
  --data fields.json \
  --output output.pdf
```

## Error Handling

The CLI will report errors for:
- Invalid JSON format
- Network failures
- Authentication failures (401/403)
- Missing template (404)
- Server errors (5xx)

Example error output:
```
Error: Failed to fetch URL: https://api.example.com/template.pdf - Status: 401 Unauthorized
```

## Best Practices

1. **Use environment variables for secrets:**
   ```bash
   TEMPLATE_URL='{"url":"https://api.example.com/template.pdf","headers":{"Authorization":"Bearer '$TOKEN'"}}'
   fill-pdf fill --template "$TEMPLATE_URL" --data fields.json --output output.pdf
   ```

2. **Store configs in files for reuse:**
   ```bash
   # template_prod.json, template_staging.json
   fill-pdf fill --template "$(cat template_prod.json)" --data fields.json --output output.pdf
   ```

3. **Use simple URLs when possible:**
   ```bash
   # Prefer this for public templates
   fill-pdf fill --template https://cdn.example.com/template.pdf --data fields.json --output output.pdf
   ```

4. **Validate JSON before use:**
   ```bash
   cat template_config.json | jq .  # Validate JSON syntax
   ```

## Security Considerations

- **Never commit tokens/keys** to version control
- **Use environment variables** for sensitive data
- **Rotate credentials** regularly
- **Use HTTPS** for all remote templates
- **Validate template sources** before processing
