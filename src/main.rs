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
