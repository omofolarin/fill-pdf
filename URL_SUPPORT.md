# URL Support for Templates and Images

## Template from URL

You can now pass a URL instead of a file path for the template:

```bash
fill-pdf fill \
  --template "https://example.com/api/templates/form.pdf" \
  --data fields.json \
  --output filled.pdf
```

## Image/Signature from URL

### Simple URL (GET request)

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

### URL with Authentication Headers

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
    "headers": {
      "Authorization": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
      "X-API-Key": "your-api-key-here"
    }
  }
}
```

### URL with POST Request

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
    "url": "https://api.example.com/generate-signature",
    "method": "POST",
    "headers": {
      "Authorization": "Bearer token",
      "Content-Type": "application/json"
    },
    "body": {
      "user_id": "123",
      "format": "png",
      "width": 150,
      "height": 50
    }
  }
}
```

### URL with Custom Headers and Query Parameters

```json
{
  "field_id": "logo",
  "page": 0,
  "x": 50.0,
  "y": 50.0,
  "width": 100.0,
  "height": 100.0,
  "field_type": "signature",
  "value": {
    "url": "https://cdn.example.com/images/logo.png?size=large&format=png",
    "headers": {
      "X-CDN-Token": "cdn-access-token",
      "Cache-Control": "no-cache"
    }
  }
}
```

## Base64 Images (Still Supported)

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

## Complete Example with Mixed Sources

```json
[
  {
    "field_id": "name",
    "page": 0,
    "x": 100.0,
    "y": 100.0,
    "width": 200.0,
    "height": 20.0,
    "field_type": "text",
    "value": "John Doe"
  },
  {
    "field_id": "company_logo",
    "page": 0,
    "x": 50.0,
    "y": 50.0,
    "width": 100.0,
    "height": 100.0,
    "field_type": "signature",
    "value": {
      "url": "https://cdn.company.com/logo.png",
      "headers": {
        "X-API-Key": "company-api-key"
      }
    }
  },
  {
    "field_id": "user_signature",
    "page": 0,
    "x": 100.0,
    "y": 500.0,
    "width": 150.0,
    "height": 50.0,
    "field_type": "signature",
    "value": {
      "url": "https://api.signatures.com/generate",
      "method": "POST",
      "headers": {
        "Authorization": "Bearer user-token",
        "Content-Type": "application/json"
      },
      "body": {
        "user_id": "123",
        "style": "cursive"
      }
    }
  },
  {
    "field_id": "stamp",
    "page": 0,
    "x": 400.0,
    "y": 500.0,
    "width": 80.0,
    "height": 80.0,
    "field_type": "signature",
    "value": "iVBORw0KGgoAAAANSUhEUgAAAAUA..."
  }
]
```

## Supported HTTP Methods

- `GET` (default)
- `POST`
- `PUT`
- `PATCH`

## Supported Image Formats

- PNG
- JPEG/JPG
- WebP
- GIF
- BMP

## Error Handling

If an image URL fails to fetch:
- A warning is printed to console
- The field is skipped
- PDF generation continues with other fields

Example output:
```
📥 Fetching template from URL...
🖼️  Fetching remote images...
  📥 Fetching image: https://api.example.com/signature.png
  ⚠️  Failed to fetch image for signature: 404 Not Found
✓ PDF filled successfully: output.pdf
```

## Security Considerations

1. **Authentication**: Always use HTTPS for sensitive data
2. **API Keys**: Store API keys securely, not in version control
3. **Token Expiry**: Ensure tokens are valid before running
4. **Rate Limiting**: Be aware of API rate limits
5. **Timeouts**: Requests timeout after 30 seconds by default

## Advanced Use Cases

### Dynamic Signature Generation

```json
{
  "field_id": "dynamic_signature",
  "page": 0,
  "x": 100.0,
  "y": 500.0,
  "width": 150.0,
  "height": 50.0,
  "field_type": "signature",
  "value": {
    "url": "https://signature-service.com/api/v1/generate",
    "method": "POST",
    "headers": {
      "Authorization": "Bearer ${SIGNATURE_API_TOKEN}",
      "Content-Type": "application/json"
    },
    "body": {
      "text": "John Doe",
      "font": "cursive",
      "color": "#000000",
      "width": 150,
      "height": 50,
      "format": "png"
    }
  }
}
```

### Fetching from S3 with Presigned URL

```json
{
  "field_id": "s3_image",
  "page": 0,
  "x": 100.0,
  "y": 300.0,
  "width": 200.0,
  "height": 100.0,
  "field_type": "signature",
  "value": {
    "url": "https://bucket.s3.amazonaws.com/images/doc.png?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=..."
  }
}
```

### Fetching with OAuth2 Token

```json
{
  "field_id": "oauth_image",
  "page": 0,
  "x": 100.0,
  "y": 400.0,
  "width": 150.0,
  "height": 75.0,
  "field_type": "signature",
  "value": {
    "url": "https://api.service.com/v1/images/123",
    "headers": {
      "Authorization": "Bearer ya29.a0AfH6SMBx...",
      "Accept": "image/png"
    }
  }
}
```
