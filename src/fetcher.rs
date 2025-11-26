use crate::types::{FieldData, FieldValue, ImageSource, UrlConfig};
use reqwest::Client;
use std::collections::HashMap;

pub async fn fetch_url_with_config(config: &UrlConfig) -> anyhow::Result<Vec<u8>> {
    let client = Client::new();
    
    let mut request = match config.method.as_deref().unwrap_or("GET") {
        "POST" => client.post(&config.url),
        "PUT" => client.put(&config.url),
        "PATCH" => client.patch(&config.url),
        _ => client.get(&config.url),
    };
    
    if let Some(headers) = &config.headers {
        for (key, value) in headers {
            request = request.header(key, value);
        }
    }
    
    if let Some(body) = &config.body {
        request = request.json(body);
    }
    
    let response = request.send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch URL: {} - Status: {}", config.url, response.status());
    }
    
    Ok(response.bytes().await?.to_vec())
}

pub async fn fetch_url(url: &str, config: Option<&UrlConfig>) -> anyhow::Result<Vec<u8>> {
    let client = Client::new();
    
    let mut request = match config.and_then(|c| c.method.as_deref()).unwrap_or("GET") {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        _ => client.get(url),
    };
    
    if let Some(cfg) = config {
        if let Some(headers) = &cfg.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }
        
        if let Some(body) = &cfg.body {
            request = request.json(body);
        }
    }
    
    let response = request.send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch URL: {} - Status: {}", url, response.status());
    }
    
    Ok(response.bytes().await?.to_vec())
}

pub async fn fetch_remote_images(fields: Vec<FieldData>) -> anyhow::Result<Vec<FieldData>> {
    let mut result = Vec::new();
    
    for mut field in fields {
        let should_fetch = match &field.value {
            FieldValue::Signature(ImageSource::Url(_)) | FieldValue::Image(ImageSource::Url(_)) => true,
            _ => false,
        };
        
        if should_fetch {
            if let FieldValue::Signature(ImageSource::Url(url_config)) | FieldValue::Image(ImageSource::Url(url_config)) = &field.value {
                println!("  📥 Fetching image: {}", url_config.url);
                match fetch_url(&url_config.url, Some(url_config)).await {
                    Ok(img_bytes) => {
                        let base64_img = base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            &img_bytes
                        );
                        field.value = match &field.value {
                            FieldValue::Signature(_) => FieldValue::Signature(ImageSource::Base64(base64_img)),
                            FieldValue::Image(_) => FieldValue::Image(ImageSource::Base64(base64_img)),
                            _ => field.value,
                        };
                    }
                    Err(e) => {
                        eprintln!("  ⚠️  Failed to fetch image for {}: {}", field.field_id, e);
                    }
                }
            }
        }
        result.push(field);
    }
    
    Ok(result)
}


pub async fn fetch_with_headers(config: &UrlConfig) -> anyhow::Result<(Vec<u8>, Option<String>, Option<String>)> {
    let client = Client::new();
    
    let mut request = match config.method.as_deref().unwrap_or("GET") {
        "POST" => client.post(&config.url),
        "PUT" => client.put(&config.url),
        "PATCH" => client.patch(&config.url),
        _ => client.get(&config.url),
    };
    
    if let Some(headers) = &config.headers {
        for (key, value) in headers {
            request = request.header(key, value);
        }
    }
    
    if let Some(body) = &config.body {
        request = request.json(body);
    }
    
    let response = request.send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch URL: {} - Status: {}", config.url, response.status());
    }
    
    let etag = response.headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    let last_modified = response.headers()
        .get("last-modified")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    
    let bytes = response.bytes().await?.to_vec();
    
    Ok((bytes, etag, last_modified))
}

pub async fn validate_cache(
    config: &UrlConfig,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> anyhow::Result<bool> {
    let client = Client::new();
    let mut request = client.head(&config.url);
    
    if let Some(headers) = &config.headers {
        for (key, value) in headers {
            request = request.header(key, value);
        }
    }
    
    if let Some(etag) = etag {
        request = request.header("If-None-Match", etag);
    }
    
    if let Some(last_modified) = last_modified {
        request = request.header("If-Modified-Since", last_modified);
    }
    
    let response = request.send().await?;
    
    // 304 Not Modified = cache is still valid
    Ok(response.status() == 304)
}
