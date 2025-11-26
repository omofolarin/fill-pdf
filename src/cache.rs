use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub template_bytes: Vec<u8>,
    pub cached_at: DateTime<Utc>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

pub struct TemplateCache {
    cache_dir: PathBuf,
    ttl_seconds: i64,
}

impl TemplateCache {
    pub fn new(cache_dir: Option<PathBuf>, ttl_seconds: Option<i64>) -> anyhow::Result<Self> {
        let cache_dir = cache_dir.unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".fill-pdf").join("cache")
        });
        
        std::fs::create_dir_all(&cache_dir)?;
        
        Ok(Self {
            cache_dir,
            ttl_seconds: ttl_seconds.unwrap_or(3600), // 1 hour default
        })
    }
    
    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        let path = self.cache_path(key);
        if !path.exists() {
            return None;
        }
        
        let data = std::fs::read(&path).ok()?;
        let entry: CacheEntry = bincode::deserialize(&data).ok()?;
        
        // Check TTL
        let age = Utc::now().signed_duration_since(entry.cached_at);
        if age > Duration::seconds(self.ttl_seconds) {
            return None;
        }
        
        Some(entry)
    }
    
    pub fn set(&self, key: &str, entry: CacheEntry) -> anyhow::Result<()> {
        let path = self.cache_path(key);
        let data = bincode::serialize(&entry)?;
        std::fs::write(path, data)?;
        Ok(())
    }
    
    pub fn clear(&self) -> anyhow::Result<()> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir)?;
            std::fs::create_dir_all(&self.cache_dir)?;
        }
        Ok(())
    }
    
    fn cache_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.cache", key))
    }
    
    pub fn generate_key(source: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
