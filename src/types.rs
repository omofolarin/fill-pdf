use serde::{Deserialize, Serialize};
use lopdf::Document;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PdfDocument {
    pub pages: Vec<PdfPageInfo>,
}

#[derive(Debug, Clone)]
pub struct PdfPageInfo {
    pub width: f32,
    pub height: f32,
    pub page_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingMetadata {
    pub pages: Vec<PageMetadata>,
    pub fields_processed: usize,
    pub fields_skipped: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageMetadata {
    pub page_number: u32,
    pub width: f32,
    pub height: f32,
    pub fields_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldData {
    pub field_id: String,
    pub page: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(flatten)]
    pub value: FieldValue,
    #[serde(default)]
    pub font_size: Option<f32>,
    #[serde(default)]
    pub alignment: Option<String>,
    #[serde(default)]
    pub vertical_alignment: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<String>>,
    #[serde(default)]
    pub fit_mode: Option<ImageFitMode>,
    #[serde(default)]
    pub text_overflow: Option<TextOverflow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextOverflow {
    /// Text can overflow beyond field boundaries (default)
    Overflow,
    /// Text is truncated at field boundaries
    Cutoff,
}

impl Default for TextOverflow {
    fn default() -> Self {
        Self::Overflow
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFitMode {
    /// Stretch to fill (may distort)
    Fill,
    /// Fit within bounds (maintain aspect ratio, may have empty space)
    Contain,
    /// Cover entire bounds (maintain aspect ratio, may crop)
    Cover,
    /// Scale down only if larger (maintain aspect ratio)
    ScaleDown,
}

impl Default for ImageFitMode {
    fn default() -> Self {
        Self::Contain
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "field_type", content = "value", rename_all = "lowercase")]
pub enum FieldValue {
    Text(String),
    Number(f64),
    Date(String),
    Checkbox(bool),
    Radio(String),
    Dropdown(String),
    Image(ImageSource),
    Signature(ImageSource),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageSource {
    Base64(String),
    Url(UrlConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateSource {
    Path(String),
    Url(UrlConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlConfig {
    pub url: String,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub body: Option<serde_json::Value>,
}

impl Default for FieldValue {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

pub fn extract_pdf_info(document: &Document) -> anyhow::Result<PdfDocument> {
    let pages = document.get_pages();
    let mut page_infos = Vec::new();
    
    for (page_num, page_id) in pages.values().enumerate() {
        // Use get_dictionary instead of get_object (like srv-ocr)
        let page_dict = document.get_dictionary(*page_id)
            .map_err(|e| anyhow::anyhow!("Failed to get page dictionary: {}", e))?;
        
        let (width, height) = if let Ok(media_box) = page_dict.get(b"MediaBox") {
            if let lopdf::Object::Array(ref arr) = *media_box {
                if arr.len() >= 4 {
                    let x1 = arr[0].as_f32().unwrap_or(0.0);
                    let y1 = arr[1].as_f32().unwrap_or(0.0);
                    let x2 = arr[2].as_f32().unwrap_or(595.0);
                    let y2 = arr[3].as_f32().unwrap_or(842.0);
                    (x2 - x1, y2 - y1)
                } else {
                    (595.0, 842.0)
                }
            } else {
                (595.0, 842.0)
            }
        } else {
            (595.0, 842.0)
        };
        
        page_infos.push(PdfPageInfo {
            width,
            height,
            page_number: page_num as u32,
        });
    }
    
    Ok(PdfDocument { pages: page_infos })
}
