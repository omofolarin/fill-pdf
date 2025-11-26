use crate::types::{FieldData, FieldValue, ImageSource, ImageFitMode, PdfDocument, ProcessingMetadata, PageMetadata};
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref, Str, Filter};
use std::collections::HashMap;

pub struct PdfFieldRenderer {
    pdf: Pdf,
    font_id: Ref,
    symbol_font_id: Ref,
    font_name: Name<'static>,
    symbol_font_name: Name<'static>,
    next_ref: i32,
    image_refs: HashMap<String, (Ref, u32, u32)>, // (ref, width, height)
    pub metadata: ProcessingMetadata,
}

impl PdfFieldRenderer {
    pub fn new() -> Self {
        Self {
            pdf: Pdf::new(),
            font_id: Ref::new(1),
            symbol_font_id: Ref::new(2),
            font_name: Name(b"F1"),
            symbol_font_name: Name(b"F2"),
            next_ref: 3,
            image_refs: HashMap::new(),
            metadata: ProcessingMetadata {
                pages: Vec::new(),
                fields_processed: 0,
                fields_skipped: 0,
                warnings: Vec::new(),
                errors: Vec::new(),
            },
        }
    }

    pub async fn create_populated_form(
        mut self,
        fields: &[FieldData],
        pdf_document: &PdfDocument,
    ) -> anyhow::Result<(Vec<u8>, ProcessingMetadata)> {
        let mut fields_by_page: HashMap<u32, Vec<&FieldData>> = HashMap::new();
        for field in fields {
            fields_by_page.entry(field.page).or_default().push(field);
        }
        
        let mut page_ids = Vec::new();
        let mut all_annotation_refs = Vec::new();
        let page_tree_id = Ref::new(self.next_ref);
        self.next_ref += 1;

        let mut sorted_pages: Vec<_> = fields_by_page.into_iter().collect();
        sorted_pages.sort_by_key(|(page_num, _)| *page_num);

        for (page_num, page_fields) in sorted_pages {
            if page_num as usize >= pdf_document.pages.len() {
                self.metadata.warnings.push(format!("Page {} not found in template", page_num));
                self.metadata.fields_skipped += page_fields.len();
                continue;
            }
            
            let page_info = &pdf_document.pages[page_num as usize];
            
            // Track page metadata
            self.metadata.pages.push(PageMetadata {
                page_number: page_num,
                width: page_info.width,
                height: page_info.height,
                fields_count: page_fields.len(),
            });
            
            let content_id = Ref::new(self.next_ref);
            self.next_ref += 1;
            let page_id = Ref::new(self.next_ref);
            self.next_ref += 1;
            
            let mut content = Content::new();
            let mut page_annotation_refs = Vec::new();
            let mut page_image_refs = Vec::new();

            for field in page_fields {
                match &field.value {
                    FieldValue::Text(_) | FieldValue::Number(_) | FieldValue::Date(_) | FieldValue::Dropdown(_) => {
                        content.begin_text();
                        self.render_text_with_fitting(field, page_info, &mut content);
                        content.end_text();
                        self.metadata.fields_processed += 1;
                    }
                    FieldValue::Checkbox(_) => {
                        let field_ref = self.create_checkbox_field(field, page_info)?;
                        page_annotation_refs.push(field_ref);
                        all_annotation_refs.push(field_ref);
                        self.metadata.fields_processed += 1;
                    }
                    FieldValue::Radio(_) => {
                        let field_ref = self.create_radio_field(field, page_info)?;
                        page_annotation_refs.push(field_ref);
                        all_annotation_refs.push(field_ref);
                        self.metadata.fields_processed += 1;
                    }
                    FieldValue::Signature(img_source) | FieldValue::Image(img_source) => {
                        let base64_img = match img_source {
                            ImageSource::Base64(b64) => b64.clone(),
                            ImageSource::Url(_) => {
                                self.metadata.warnings.push(format!("Skipped URL image for field {}", field.field_id));
                                self.metadata.fields_skipped += 1;
                                continue;
                            }
                        };
                        match self.decode_image(&base64_img) {
                            Ok(img_data) => {
                                match self.embed_image(&img_data, &field.field_id) {
                                    Ok((img_ref, _, _)) => {
                                        page_image_refs.push(img_ref);
                                        self.render_embedded_image(field, page_info, img_ref, &mut content);
                                        self.metadata.fields_processed += 1;
                                    }
                                    Err(e) => {
                                        self.metadata.errors.push(format!("Failed to embed image {}: {}", field.field_id, e));
                                        self.metadata.fields_skipped += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                self.metadata.errors.push(format!("Failed to decode image {}: {}", field.field_id, e));
                                self.metadata.fields_skipped += 1;
                            }
                        }
                    }
                }
            }
            
            let mut page = self.pdf.page(page_id);
            page.media_box(Rect::new(0.0, 0.0, page_info.width, page_info.height))
                .parent(page_tree_id)
                .contents(content_id);
            
            if !page_annotation_refs.is_empty() {
                page.annotations(page_annotation_refs.iter().copied());
            }
            
            {
                let mut resources = page.resources();
                resources.fonts()
                    .pair(self.font_name, self.font_id)
                    .pair(self.symbol_font_name, self.symbol_font_id);
                
                if !page_image_refs.is_empty() {
                    let mut xobjects = resources.x_objects();
                    for img_ref in &page_image_refs {
                        let img_name = format!("Im{}", img_ref.get());
                        xobjects.pair(Name(img_name.as_bytes()), *img_ref);
                    }
                }
            }
            
            page.finish();
            
            self.pdf.stream(content_id, &content.finish());
            page_ids.push(page_id);
        }
        
        let catalog_id = Ref::new(self.next_ref);
        self.next_ref += 1;

        self.pdf.type1_font(self.font_id)
            .base_font(Name(b"Helvetica"));
        self.pdf.type1_font(self.symbol_font_id)
            .base_font(Name(b"ZapfDingbats"));
        
        let page_count = page_ids.len() as i32;
        self.pdf.pages(page_tree_id).kids(page_ids).count(page_count);
        
        let mut cat = self.pdf.catalog(catalog_id);
        cat.pages(page_tree_id);
        if !all_annotation_refs.is_empty() {
            cat.form().fields(all_annotation_refs.iter().copied());
        }
        cat.finish();

        Ok((self.pdf.finish(), self.metadata))
    }

    fn render_text_with_fitting(&self, field: &FieldData, page_info: &crate::types::PdfPageInfo, content: &mut Content) {
        let (pdf_x, pdf_y, width, height) = self.convert_coordinates(field, page_info);
        
        let text = match &field.value {
            FieldValue::Text(t) => t.clone(),
            FieldValue::Number(n) => n.to_string(),
            FieldValue::Date(d) => d.clone(),
            FieldValue::Dropdown(d) => d.clone(),
            _ => return,
        };

        if text.is_empty() {
            return;
        }

        content.set_fill_rgb(0.0, 0.0, 0.0);

        let base_font_size = field.font_size.unwrap_or(12.0).max(12.0);
        let reduced_font_size = base_font_size * 0.9;
        
        let base_y = if height > base_font_size * 1.2 {
            match field.vertical_alignment.as_deref() {
                Some("middle") => pdf_y + (height - base_font_size) / 2.0,
                Some("bottom") => pdf_y + height - base_font_size,
                Some("baseline") => pdf_y + height - (base_font_size * 0.2),
                _ => pdf_y,
            }
        } else {
            pdf_y
        };
        
        let char_width = base_font_size * 0.5;
        let text_width = text.len() as f32 * char_width;
        
        if text_width <= width {
            let x_offset = match field.alignment.as_deref() {
                Some("center") => (width - text_width) / 2.0,
                Some("right") => width - text_width,
                _ => 0.0,
            };
            
            content.set_font(self.font_name, base_font_size);
            content.next_line(pdf_x + x_offset, base_y);
            content.show(Str(text.as_bytes()));
            return;
        }
        
        let reduced_char_width = reduced_font_size * 0.5;
        let reduced_text_width = text.len() as f32 * reduced_char_width;
        
        if reduced_text_width <= width {
            let x_offset = match field.alignment.as_deref() {
                Some("center") => (width - reduced_text_width) / 2.0,
                Some("right") => width - reduced_text_width,
                _ => 0.0,
            };
            
            let reduced_y = if height > reduced_font_size * 1.2 {
                match field.vertical_alignment.as_deref() {
                    Some("middle") => pdf_y + (height - reduced_font_size) / 2.0,
                    Some("bottom") => pdf_y + height - reduced_font_size,
                    Some("baseline") => pdf_y + height - (reduced_font_size * 0.2),
                    _ => pdf_y,
                }
            } else {
                pdf_y
            };
            
            content.set_font(self.font_name, reduced_font_size);
            content.next_line(pdf_x + x_offset, reduced_y);
            content.show(Str(text.as_bytes()));
            return;
        }
        
        let chars_per_line = (width / char_width).floor() as usize;
        
        if chars_per_line > 0 {
            let line_height = base_font_size * 1.2;
            let max_lines = (height / line_height).floor() as usize;
            
            let mut lines = Vec::new();
            let mut remaining = text.as_str();
            
            while !remaining.is_empty() && lines.len() < max_lines {
                let split_at = remaining.char_indices()
                    .nth(chars_per_line)
                    .map(|(i, _)| i)
                    .unwrap_or(remaining.len());
                
                let (line, rest) = remaining.split_at(split_at);
                lines.push(line);
                remaining = rest;
            }
            
            if remaining.is_empty() {
                content.set_font(self.font_name, base_font_size);
                
                let total_text_height = lines.len() as f32 * line_height;
                let first_line_y = if height > total_text_height {
                    match field.vertical_alignment.as_deref() {
                        Some("middle") => pdf_y + (height - total_text_height) / 2.0,
                        Some("bottom") | Some("baseline") => pdf_y + height - total_text_height,
                        _ => pdf_y,
                    }
                } else {
                    pdf_y
                };
                
                for (i, line) in lines.iter().enumerate() {
                    let line_width = line.len() as f32 * char_width;
                    let x_offset = match field.alignment.as_deref() {
                        Some("center") => (width - line_width) / 2.0,
                        Some("right") => width - line_width,
                        _ => 0.0,
                    };
                    
                    let y_offset = first_line_y + (i as f32 * line_height);
                    content.next_line(pdf_x + x_offset, y_offset);
                    content.show(Str(line.as_bytes()));
                }
                return;
            }
        }
        
        content.set_font(self.font_name, base_font_size);
        content.next_line(pdf_x, pdf_y);
        content.show(Str(text.as_bytes()));
    }

    fn create_checkbox_field(&mut self, field: &FieldData, page_info: &crate::types::PdfPageInfo) -> anyhow::Result<Ref> {
        let field_id = Ref::new(self.next_ref);
        self.next_ref += 1;
        let on_appearance_id = Ref::new(self.next_ref);
        self.next_ref += 1;
        let off_appearance_id = Ref::new(self.next_ref);
        self.next_ref += 1;

        let (pdf_x, pdf_y, width, height) = self.convert_coordinates(field, page_info);
        let bbox = Rect::new(0.0, 0.0, width, height);

        let mut content = Content::new();
        content.begin_text();
        content.set_font(self.symbol_font_name, 14.0);
        content.show(Str(b"4")); // Checkmark
        content.end_text();

        let content_data = content.finish();
        let mut on_appearance = self.pdf.form_xobject(on_appearance_id, &content_data);
        on_appearance.bbox(bbox);
        on_appearance.resources().fonts().pair(self.symbol_font_name, self.symbol_font_id);
        on_appearance.finish();

        self.pdf.form_xobject(off_appearance_id, &Content::new().finish()).bbox(bbox);

        let mut pdf_field = self.pdf.form_field(field_id);
        pdf_field
            .partial_name(pdf_writer::TextStr(&field.field_id))
            .field_type(pdf_writer::types::FieldType::Button);

        let mut annot = pdf_field.into_annotation();
        annot.rect(Rect::new(pdf_x, pdf_y, pdf_x + width, pdf_y + height));
        
        let is_checked = matches!(field.value, FieldValue::Checkbox(true));
        annot.appearance_state(if is_checked { Name(b"Yes") } else { Name(b"Off") });
        annot.flags(pdf_writer::types::AnnotationFlags::PRINT);

        let mut appearance = annot.appearance();
        appearance.normal().streams().pairs([
            (Name(b"Yes"), on_appearance_id),
            (Name(b"Off"), off_appearance_id),
        ]);
        
        Ok(field_id)
    }

    fn create_radio_field(&mut self, field: &FieldData, page_info: &crate::types::PdfPageInfo) -> anyhow::Result<Ref> {
        let field_id = Ref::new(self.next_ref);
        self.next_ref += 1;
        let on_appearance_id = Ref::new(self.next_ref);
        self.next_ref += 1;
        let off_appearance_id = Ref::new(self.next_ref);
        self.next_ref += 1;

        let (pdf_x, pdf_y, width, height) = self.convert_coordinates(field, page_info);
        let bbox = Rect::new(0.0, 0.0, width, height);

        let mut content = Content::new();
        content.begin_text();
        content.set_font(self.symbol_font_name, 14.0);
        content.show(Str(b"l")); // Filled circle
        content.end_text();

        let content_data = content.finish();
        let mut on_appearance = self.pdf.form_xobject(on_appearance_id, &content_data);
        on_appearance.bbox(bbox);
        on_appearance.resources().fonts().pair(self.symbol_font_name, self.symbol_font_id);
        on_appearance.finish();

        self.pdf.form_xobject(off_appearance_id, &Content::new().finish()).bbox(bbox);

        let mut pdf_field = self.pdf.form_field(field_id);
        pdf_field
            .partial_name(pdf_writer::TextStr(&field.field_id))
            .field_type(pdf_writer::types::FieldType::Button)
            .field_flags(pdf_writer::types::FieldFlags::RADIO | pdf_writer::types::FieldFlags::NO_TOGGLE_TO_OFF);

        let mut annot = pdf_field.into_annotation();
        annot.rect(Rect::new(pdf_x, pdf_y, pdf_x + width, pdf_y + height));
        
        let is_selected = matches!(field.value, FieldValue::Radio(_));
        annot.appearance_state(if is_selected { Name(b"Yes") } else { Name(b"Off") });
        annot.flags(pdf_writer::types::AnnotationFlags::PRINT);

        let mut appearance = annot.appearance();
        appearance.normal().streams().pairs([
            (Name(b"Yes"), on_appearance_id),
            (Name(b"Off"), off_appearance_id),
        ]);
        
        Ok(field_id)
    }

    fn decode_image(&self, base64_str: &str) -> anyhow::Result<Vec<u8>> {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        Ok(STANDARD.decode(base64_str)?)
    }

    fn embed_image(&mut self, img_data: &[u8], field_id: &str) -> anyhow::Result<(Ref, u32, u32)> {
        if let Some(&existing) = self.image_refs.get(field_id) {
            return Ok(existing);
        }
        
        let img = image::load_from_memory(img_data)?;
        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();
        
        let image_id = Ref::new(self.next_ref);
        self.next_ref += 1;
        
        let mut image = self.pdf.image_xobject(image_id, rgb_img.as_raw());
        image.width(width as i32);
        image.height(height as i32);
        image.color_space().device_rgb();
        image.bits_per_component(8);
        image.filter(Filter::DctDecode);
        image.finish();
        
        self.image_refs.insert(field_id.to_string(), (image_id, width, height));
        Ok((image_id, width, height))
    }

    fn render_embedded_image(&self, field: &FieldData, page_info: &crate::types::PdfPageInfo, img_ref: Ref, content: &mut Content) {
        let (pdf_x, pdf_y, box_width, box_height) = self.convert_coordinates(field, page_info);
        
        // Get actual image dimensions
        let (img_width, img_height) = self.image_refs.get(&field.field_id)
            .map(|(_, w, h)| (*w as f32, *h as f32))
            .unwrap_or((box_width, box_height));
        
        let fit_mode = field.fit_mode.as_ref().unwrap_or(&ImageFitMode::Contain);
        
        let (render_width, render_height, offset_x, offset_y) = match fit_mode {
            ImageFitMode::Fill => {
                (box_width, box_height, 0.0, 0.0)
            }
            ImageFitMode::Contain => {
                let scale = (box_width / img_width).min(box_height / img_height);
                let w = img_width * scale;
                let h = img_height * scale;
                (w, h, (box_width - w) / 2.0, (box_height - h) / 2.0)
            }
            ImageFitMode::Cover => {
                let scale = (box_width / img_width).max(box_height / img_height);
                let w = img_width * scale;
                let h = img_height * scale;
                (w, h, (box_width - w) / 2.0, (box_height - h) / 2.0)
            }
            ImageFitMode::ScaleDown => {
                if img_width <= box_width && img_height <= box_height {
                    (img_width, img_height, (box_width - img_width) / 2.0, (box_height - img_height) / 2.0)
                } else {
                    let scale = (box_width / img_width).min(box_height / img_height);
                    let w = img_width * scale;
                    let h = img_height * scale;
                    (w, h, (box_width - w) / 2.0, (box_height - h) / 2.0)
                }
            }
        };
        
        content.save_state();
        content.transform([render_width, 0.0, 0.0, render_height, pdf_x + offset_x, pdf_y + offset_y]);
        content.x_object(Name(format!("Im{}", img_ref.get()).as_bytes()));
        content.restore_state();
    }

    fn convert_coordinates(&self, field: &FieldData, page_info: &crate::types::PdfPageInfo) -> (f32, f32, f32, f32) {
        let x = field.x;
        let y = field.y;
        let width = field.width;
        let height = field.height;
        
        let pdf_y = page_info.height - y - height;
        
        (x, pdf_y, width, height)
    }
}
