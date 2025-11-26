use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod renderer;
mod types;
mod merge;
mod fetcher;
mod cache;

use renderer::PdfFieldRenderer;
use types::{PdfDocument, FieldData, TemplateSource};

#[derive(Parser)]
#[command(name = "fill-pdf")]
#[command(about = "Fill PDF forms with data", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fill a PDF template with JSON data
    Fill {
        /// Path or URL to template PDF
        #[arg(short, long)]
        template: String,
        
        /// Path to JSON data file
        #[arg(short, long)]
        data: PathBuf,
        
        /// Output PDF path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Output metadata JSON path (optional)
        #[arg(short, long)]
        metadata: Option<PathBuf>,
        
        /// Enable template caching
        #[arg(long)]
        cache: bool,
        
        /// Cache directory (default: ~/.fill-pdf/cache)
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        
        /// Cache TTL in seconds (default: 3600)
        #[arg(long)]
        cache_ttl: Option<i64>,
        
        /// Force cache refresh
        #[arg(long)]
        cache_refresh: bool,
        
        /// Keep interactive form fields (default: flatten)
        #[arg(long)]
        keep_fields: bool,
        
        /// Merge backend: python (PyPDF2) or bun (pdf-lib)
        #[arg(long, default_value = "python")]
        merge_backend: String,
        
        /// Text overflow mode: overflow (default) or cutoff
        #[arg(long, default_value = "overflow")]
        text_overflow: String,
    },
    
    /// Convert PDF pages to images (PNG/JPEG)
    ToImage {
        /// Input PDF file(s)
        #[arg(required = true)]
        pdfs: Vec<PathBuf>,
        
        /// Output directory for images
        #[arg(short, long)]
        output_dir: Option<PathBuf>,
        
        /// Image format: png or jpeg
        #[arg(short, long, default_value = "png")]
        format: String,
        
        /// DPI resolution (default: 300)
        #[arg(long, default_value = "300")]
        dpi: u32,
        
        /// Specific pages to convert (e.g., "1,3,5" or "1-3,5")
        #[arg(short, long)]
        pages: Option<String>,
        
        /// Output as base64 encoded strings (prints to stdout)
        #[arg(long)]
        base64: bool,
    },
    
    /// Cache management
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
}

#[derive(Subcommand)]
enum CacheCommands {
    /// Clear template cache
    Clear {
        /// Cache directory (default: ~/.fill-pdf/cache)
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fill { template, data, output, metadata, cache, cache_dir, cache_ttl, cache_refresh, keep_fields, merge_backend, text_overflow } => {
            fill_pdf(template, data, output, metadata, cache, cache_dir, cache_ttl, cache_refresh, keep_fields, merge_backend, text_overflow).await?;
        }
        Commands::ToImage { pdfs, output_dir, format, dpi, pages, base64 } => {
            pdf_to_images(pdfs, output_dir, format, dpi, pages, base64).await?;
        }
        Commands::Cache { command } => {
            match command {
                CacheCommands::Clear { cache_dir } => {
                    let cache = cache::TemplateCache::new(cache_dir, None)?;
                    cache.clear()?;
                    println!("✓ Cache cleared");
                }
            }
        }
    }

    Ok(())
}

async fn fill_pdf(
    template: String, 
    data: PathBuf, 
    output: PathBuf, 
    metadata_path: Option<PathBuf>,
    use_cache: bool,
    cache_dir: Option<PathBuf>,
    cache_ttl: Option<i64>,
    cache_refresh: bool,
    keep_fields: bool,
    merge_backend: String,
    text_overflow: String,
) -> anyhow::Result<()> {
    // Check dependencies first
    merge::ensure_dependencies(&merge_backend)?;
    
    // Parse template source
    let template_source: TemplateSource = if template.starts_with('{') {
        serde_json::from_str(&template)?
    } else if template.starts_with("http://") || template.starts_with("https://") {
        TemplateSource::Url(types::UrlConfig {
            url: template.clone(),
            method: None,
            headers: None,
            body: None,
        })
    } else {
        TemplateSource::Path(template.clone())
    };
    
    // Load template bytes (with caching if enabled)
    let template_bytes = if use_cache && !matches!(template_source, TemplateSource::Path(_)) {
        let cache = cache::TemplateCache::new(cache_dir, cache_ttl)?;
        let cache_key = cache::TemplateCache::generate_key(&template);
        
        if cache_refresh {
            println!("🔄 Forcing cache refresh...");
            fetch_and_cache_template(&template_source, &cache, &cache_key).await?
        } else if let Some(entry) = cache.get(&cache_key) {
            println!("✓ Using cached template");
            
            // Validate with server if we have ETag/Last-Modified
            if entry.etag.is_some() || entry.last_modified.is_some() {
                match validate_cache(&template_source, &entry).await {
                    Ok(true) => entry.template_bytes,
                    Ok(false) => {
                        println!("🔄 Template updated, refreshing cache...");
                        fetch_and_cache_template(&template_source, &cache, &cache_key).await?
                    }
                    Err(_) => {
                        println!("⚠️  Cache validation failed, using cached version");
                        entry.template_bytes
                    }
                }
            } else {
                entry.template_bytes
            }
        } else {
            println!("📥 Fetching and caching template...");
            fetch_and_cache_template(&template_source, &cache, &cache_key).await?
        }
    } else {
        match template_source {
            TemplateSource::Path(path) => std::fs::read(&path)?,
            TemplateSource::Url(url_config) => {
                println!("📥 Fetching template from URL...");
                fetcher::fetch_url_with_config(&url_config).await?
            }
        }
    };
    
    // Load PDF document using Cursor (same as srv-ocr)
    let template_doc = lopdf::Document::load_from(std::io::Cursor::new(&template_bytes))
        .map_err(|e| anyhow::anyhow!("Failed to load PDF document: {}", e))?;
    let pdf_info = types::extract_pdf_info(&template_doc)?;
    
    // Load field data
    let json_data = std::fs::read_to_string(&data)?;
    let mut field_data: Vec<FieldData> = serde_json::from_str(&json_data)?;
    
    // Apply global text_overflow to fields without explicit setting
    let global_overflow = match text_overflow.as_str() {
        "cutoff" => types::TextOverflow::Cutoff,
        _ => types::TextOverflow::Overflow,
    };
    
    for field in &mut field_data {
        if field.text_overflow.is_none() {
            field.text_overflow = Some(global_overflow.clone());
        }
    }
    
    // Fetch remote images/signatures
    println!("🖼️  Fetching remote images...");
    let field_data = fetcher::fetch_remote_images(field_data).await?;
    
    // Create renderer and fill
    let renderer = PdfFieldRenderer::new();
    let (filled_pdf, metadata) = renderer.create_populated_form(&field_data, &pdf_info).await?;
    
    // Merge with template
    let final_pdf = merge::merge_pdfs_bytes(&template_bytes, &filled_pdf, !keep_fields, &merge_backend)?;
    
    // Save output
    std::fs::write(&output, final_pdf)?;
    
    println!("✓ PDF filled successfully: {}", output.display());
    println!("  Fields processed: {}", metadata.fields_processed);
    println!("  Fields skipped: {}", metadata.fields_skipped);
    
    if !metadata.warnings.is_empty() {
        println!("⚠️  Warnings:");
        for warning in &metadata.warnings {
            println!("    - {}", warning);
        }
    }
    
    if !metadata.errors.is_empty() {
        println!("❌ Errors:");
        for error in &metadata.errors {
            println!("    - {}", error);
        }
    }
    
    // Save metadata if requested
    if let Some(meta_path) = metadata_path {
        let meta_json = serde_json::to_string_pretty(&metadata)?;
        std::fs::write(&meta_path, meta_json)?;
        println!("📊 Metadata saved: {}", meta_path.display());
    }
    
    Ok(())
}


async fn fetch_and_cache_template(
    source: &TemplateSource,
    cache: &cache::TemplateCache,
    cache_key: &str,
) -> anyhow::Result<Vec<u8>> {
    let (bytes, etag, last_modified) = match source {
        TemplateSource::Path(_) => unreachable!(),
        TemplateSource::Url(config) => fetcher::fetch_with_headers(&config).await?,
    };
    
    let entry = cache::CacheEntry {
        template_bytes: bytes.clone(),
        cached_at: chrono::Utc::now(),
        etag,
        last_modified,
    };
    
    cache.set(cache_key, entry)?;
    Ok(bytes)
}

async fn validate_cache(
    source: &TemplateSource,
    entry: &cache::CacheEntry,
) -> anyhow::Result<bool> {
    match source {
        TemplateSource::Path(_) => Ok(true),
        TemplateSource::Url(config) => {
            fetcher::validate_cache(&config, entry.etag.as_deref(), entry.last_modified.as_deref()).await
        }
    }
}

async fn pdf_to_images(
    pdfs: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
    format: String,
    dpi: u32,
    pages: Option<String>,
    base64: bool,
) -> anyhow::Result<()> {
    use std::process::Command;
    
    // Validate format
    let format_lower = format.to_lowercase();
    if format_lower != "png" && format_lower != "jpeg" && format_lower != "jpg" {
        anyhow::bail!("Invalid format: {}. Use 'png' or 'jpeg'", format);
    }
    
    let img_format = if format_lower == "jpg" { "jpeg" } else { &format_lower };
    
    // Validate output_dir if not base64
    if !base64 && output_dir.is_none() {
        anyhow::bail!("--output-dir is required when not using --base64");
    }
    
    // Check and install dependencies
    check_pdf_to_image_deps().await?;
    
    // Create output directory if needed
    if let Some(ref dir) = output_dir {
        std::fs::create_dir_all(dir)?;
    }
    
    if !base64 {
        println!("🖼️  Converting {} PDF(s) to {} images at {} DPI...", pdfs.len(), format.to_uppercase(), dpi);
    }
    
    // Parse page selection if provided
    let page_list = if let Some(ref pages_str) = pages {
        Some(parse_page_selection(pages_str)?)
    } else {
        None
    };
    
    for pdf_path in pdfs {
        if !pdf_path.exists() {
            eprintln!("⚠️  Skipping non-existent file: {}", pdf_path.display());
            continue;
        }
        
        let pdf_name = pdf_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        
        if !base64 {
            println!("  📄 Processing: {}", pdf_path.display());
        }
        
        let pdf_str = pdf_path.display().to_string();
        
        // Build page selection parameter for Python
        let page_param = if let Some(ref pages) = page_list {
            format!("first_page={}, last_page={}", 
                pages.iter().min().unwrap_or(&1),
                pages.iter().max().unwrap_or(&999))
        } else {
            String::new()
        };
        
        // Build Python script
        let script = if base64 {
            format!(
                r#"
from pdf2image import convert_from_path
import base64
import io
import sys
import json

try:
    images = convert_from_path('{}', dpi={}, fmt='{}'{})
    result = []
    page_filter = {}
    for i, image in enumerate(images):
        page_num = i + 1
        if page_filter is None or page_num in page_filter:
            buffer = io.BytesIO()
            image.save(buffer, format='{}')
            b64 = base64.b64encode(buffer.getvalue()).decode('utf-8')
            result.append({{'page': page_num, 'data': b64}})
    print(json.dumps(result))
except Exception as e:
    print('✗ Error: {{}}'.format(e), file=sys.stderr)
    sys.exit(1)
"#,
                pdf_str,
                dpi,
                img_format,
                if !page_param.is_empty() { format!(", {}", page_param) } else { String::new() },
                if let Some(ref pages) = page_list {
                    format!("{:?}", pages)
                } else {
                    "None".to_string()
                },
                format.to_uppercase()
            )
        } else {
            let output_str = output_dir.as_ref().unwrap().display().to_string();
            format!(
                r#"
from pdf2image import convert_from_path
import sys

try:
    images = convert_from_path('{}', dpi={}, fmt='{}'{})
    page_filter = {}
    saved = 0
    for i, image in enumerate(images):
        page_num = i + 1
        if page_filter is None or page_num in page_filter:
            output_path = '{}/{}_{{:03d}}.{}'.format(page_num)
            image.save(output_path, '{}', optimize=True)
            saved += 1
    print('✓ Converted {{}} page(s)'.format(saved))
except Exception as e:
    print('✗ Error: {{}}'.format(e), file=sys.stderr)
    sys.exit(1)
"#,
                pdf_str,
                dpi,
                img_format,
                if !page_param.is_empty() { format!(", {}", page_param) } else { String::new() },
                if let Some(ref pages) = page_list {
                    format!("{:?}", pages)
                } else {
                    "None".to_string()
                },
                output_str,
                pdf_name,
                img_format,
                format.to_uppercase()
            )
        };
        
        let output = Command::new("python3")
            .arg("-c")
            .arg(&script)
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("  ✗ Failed: {}", error);
            continue;
        }
        
        if base64 {
            // Print base64 output directly
            print!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            print!("  {}", String::from_utf8_lossy(&output.stdout));
        }
    }
    
    if !base64 {
        println!("\n✅ Images saved to: {}", output_dir.unwrap().display());
    }
    
    Ok(())
}

async fn check_pdf_to_image_deps() -> anyhow::Result<()> {
    use std::process::Command;
    
    // Check if pdf2image is installed
    let check_pdf2image = Command::new("python3")
        .arg("-c")
        .arg("import pdf2image")
        .output()?;
    
    if !check_pdf2image.status.success() {
        println!("📦 Installing pdf2image...");
        let install = Command::new("pip3")
            .args(&["install", "pdf2image", "--break-system-packages"])
            .output()?;
        
        if !install.status.success() {
            anyhow::bail!("Failed to install pdf2image. Please run: pip3 install pdf2image");
        }
        println!("✓ pdf2image installed");
    }
    
    // Check if poppler is installed
    let check_poppler = Command::new("which")
        .arg("pdftoppm")
        .output()?;
    
    if !check_poppler.status.success() {
        println!("📦 Installing poppler...");
        
        // Detect OS and install poppler
        #[cfg(target_os = "macos")]
        {
            let install = Command::new("brew")
                .args(&["install", "poppler"])
                .output()?;
            
            if !install.status.success() {
                anyhow::bail!("Failed to install poppler. Please run: brew install poppler");
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // Try apt-get first (Ubuntu/Debian)
            let install = Command::new("sudo")
                .args(&["apt-get", "install", "-y", "poppler-utils"])
                .output();
            
            if install.is_err() || !install.unwrap().status.success() {
                // Try yum (RHEL/CentOS)
                let install = Command::new("sudo")
                    .args(&["yum", "install", "-y", "poppler-utils"])
                    .output()?;
                
                if !install.status.success() {
                    anyhow::bail!("Failed to install poppler. Please run: sudo apt-get install poppler-utils");
                }
            }
        }
        
        println!("✓ poppler installed");
    }
    
    Ok(())
}

/// Parse page selection string (e.g., "1,3,5" or "1-3,5")
fn parse_page_selection(pages_str: &str) -> anyhow::Result<Vec<usize>> {
    let mut pages = Vec::new();
    
    for part in pages_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        
        if part.contains('-') {
            // Range: "1-3"
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() != 2 {
                anyhow::bail!("Invalid page range: {}", part);
            }
            
            let start: usize = range_parts[0].parse()
                .map_err(|_| anyhow::anyhow!("Invalid page number: {}", range_parts[0]))?;
            let end: usize = range_parts[1].parse()
                .map_err(|_| anyhow::anyhow!("Invalid page number: {}", range_parts[1]))?;
            
            if start > end {
                anyhow::bail!("Invalid range: {} (start > end)", part);
            }
            
            for page in start..=end {
                pages.push(page);
            }
        } else {
            // Single page: "5"
            let page: usize = part.parse()
                .map_err(|_| anyhow::anyhow!("Invalid page number: {}", part))?;
            pages.push(page);
        }
    }
    
    pages.sort();
    pages.dedup();
    
    Ok(pages)
}
