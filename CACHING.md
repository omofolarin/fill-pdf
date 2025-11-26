# Template Caching

## Overview

The CLI supports optional template caching with TTL-based expiry and ETag/Last-Modified validation for optimal performance and freshness.

## Quick Start

```bash
# Enable caching (1 hour TTL by default)
fill-pdf fill --template https://example.com/template.pdf --cache -d fields.json -o output.pdf

# First run: Fetches and caches
# Subsequent runs: Uses cache (instant)
```

## How It Works

### Cache Strategy

1. **Check cache** - Look for cached template
2. **Check TTL** - If within TTL, proceed to validation
3. **Validate** - Send HEAD request with If-None-Match/If-Modified-Since
4. **304 Not Modified** - Use cache, update timestamp
5. **200 OK** - Download new template, update cache

### Cache Key

SHA256 hash of template source (URL + headers + body)
- Same URL = same cache entry
- Different headers/body = different cache entry

### Cache Location

Default: `~/.fill-pdf/cache/`
Custom: `--cache-dir /path/to/cache`

## Usage

### Basic Caching

```bash
# Enable with default settings (1 hour TTL)
fill-pdf fill -t https://example.com/template.pdf --cache -d fields.json -o output.pdf
```

### Custom TTL

```bash
# 5 minutes
fill-pdf fill -t URL --cache --cache-ttl 300 -d fields.json -o output.pdf

# 24 hours
fill-pdf fill -t URL --cache --cache-ttl 86400 -d fields.json -o output.pdf

# 7 days
fill-pdf fill -t URL --cache --cache-ttl 604800 -d fields.json -o output.pdf
```

### Force Refresh

```bash
# Bypass cache, fetch fresh template
fill-pdf fill -t URL --cache --cache-refresh -d fields.json -o output.pdf
```

### Custom Cache Directory

```bash
# Use project-specific cache
fill-pdf fill -t URL --cache --cache-dir ./cache -d fields.json -o output.pdf
```

### Clear Cache

```bash
# Clear default cache
fill-pdf cache clear

# Clear custom cache
fill-pdf cache clear --cache-dir ./cache
```

## Cache Validation

### ETag Support

If server provides `ETag` header:
```
1. Cache stores ETag
2. Validation sends: If-None-Match: "etag-value"
3. Server responds:
   - 304 Not Modified → Use cache
   - 200 OK → Update cache
```

### Last-Modified Support

If server provides `Last-Modified` header:
```
1. Cache stores Last-Modified timestamp
2. Validation sends: If-Modified-Since: "timestamp"
3. Server responds:
   - 304 Not Modified → Use cache
   - 200 OK → Update cache
```

### Fallback

If server doesn't support validation:
- Relies on TTL only
- Fetches new template after TTL expires

## Use Cases

### Batch Processing

```bash
# Process 100 documents with same template
for i in {1..100}; do
  fill-pdf fill -t URL --cache -d data_$i.json -o output_$i.pdf
done

# First run: Fetches template
# Runs 2-100: Use cache (instant)
```

### CI/CD Pipelines

```bash
# Cache template across pipeline steps
- name: Generate PDFs
  run: |
    fill-pdf fill -t $TEMPLATE_URL --cache --cache-dir ./cache -d fields.json -o output.pdf
```

### Development Workflow

```bash
# Long TTL for stable templates
fill-pdf fill -t URL --cache --cache-ttl 604800 -d fields.json -o output.pdf

# Force refresh when template changes
fill-pdf fill -t URL --cache --cache-refresh -d fields.json -o output.pdf
```

### Multi-Environment

```bash
# Separate caches per environment
fill-pdf fill -t $PROD_URL --cache --cache-dir ~/.fill-pdf/cache/prod -d fields.json -o output.pdf
fill-pdf fill -t $STAGING_URL --cache --cache-dir ~/.fill-pdf/cache/staging -d fields.json -o output.pdf
```

## Console Output

### Cache Hit
```
✓ Using cached template
✓ PDF filled successfully: output.pdf
```

### Cache Miss
```
📥 Fetching and caching template...
✓ PDF filled successfully: output.pdf
```

### Cache Refresh
```
🔄 Template updated, refreshing cache...
✓ PDF filled successfully: output.pdf
```

### Validation Failure
```
⚠️  Cache validation failed, using cached version
✓ PDF filled successfully: output.pdf
```

## Performance

### Without Cache
```
Run 1: 2.5s (fetch + process)
Run 2: 2.5s (fetch + process)
Run 3: 2.5s (fetch + process)
```

### With Cache
```
Run 1: 2.5s (fetch + cache + process)
Run 2: 0.3s (cache + process) - 8x faster
Run 3: 0.3s (cache + process) - 8x faster
```

### With Validation (304)
```
Run 1: 2.5s (fetch + cache + process)
Run 2: 0.4s (validate + cache + process) - 6x faster
Run 3: 0.4s (validate + cache + process) - 6x faster
```

## Cache Storage

### File Format

Binary format (bincode) containing:
```rust
{
  template_bytes: Vec<u8>,
  cached_at: DateTime<Utc>,
  etag: Option<String>,
  last_modified: Option<String>,
}
```

### File Naming

`{sha256_hash}.cache`

Example: `a3f5b2c1d4e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t1u2v3w4x5y6z7.cache`

## Best Practices

1. **Use caching for remote templates** - Local files don't need caching
2. **Set appropriate TTL** - Balance freshness vs. performance
3. **Use validation** - Servers with ETag/Last-Modified support
4. **Clear cache periodically** - Prevent stale data accumulation
5. **Separate caches** - Use different cache dirs for different environments

## Limitations

1. **Local files not cached** - Only URL-based templates
2. **No size limits** - Cache can grow indefinitely
3. **No automatic cleanup** - Manual `cache clear` required
4. **Binary format** - Not human-readable

## Security Considerations

- Cache files contain full template bytes
- Cache directory should have restricted permissions
- Sensitive templates should use short TTL
- Clear cache when credentials change

## Troubleshooting

### Cache not working
```bash
# Check cache directory exists
ls -la ~/.fill-pdf/cache/

# Try with explicit cache dir
fill-pdf fill -t URL --cache --cache-dir ./test-cache -d fields.json -o output.pdf
```

### Stale cache
```bash
# Force refresh
fill-pdf fill -t URL --cache --cache-refresh -d fields.json -o output.pdf

# Or clear cache
fill-pdf cache clear
```

### Disk space issues
```bash
# Check cache size
du -sh ~/.fill-pdf/cache/

# Clear cache
fill-pdf cache clear
```

## Future Enhancements

Potential improvements:
- Cache size limits
- Automatic cleanup (LRU eviction)
- Cache statistics
- Compression
- Shared cache across users
