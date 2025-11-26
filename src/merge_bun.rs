use std::process::Command;
use std::path::PathBuf;

pub fn merge_with_bun(template_bytes: &[u8], overlay_pdf: &[u8], flatten: bool) -> anyhow::Result<Vec<u8>> {
    let temp_dir = std::env::temp_dir();
    let temp_template = temp_dir.join("fill_pdf_template.pdf");
    let temp_overlay = temp_dir.join("fill_pdf_overlay.pdf");
    let temp_merged = temp_dir.join("fill_pdf_merged.pdf");
    
    std::fs::write(&temp_template, template_bytes)?;
    std::fs::write(&temp_overlay, overlay_pdf)?;
    
    let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("merge_pdfs.ts");
    
    let mut cmd = Command::new("bun");
    cmd.arg("run")
        .arg(&script_path)
        .arg("--template").arg(&temp_template)
        .arg("--overlay").arg(&temp_overlay)
        .arg("--output").arg(&temp_merged);
    
    if flatten {
        cmd.arg("--flatten");
    }
    
    let output = cmd.output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Bun merge failed: {}", stderr);
    }
    
    // Extract timing from output
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(timing) = stdout.strip_prefix("SUCCESS:") {
        println!("  (Bun: {})", timing.trim());
    }
    
    let merged = std::fs::read(&temp_merged)?;
    
    let _ = std::fs::remove_file(&temp_template);
    let _ = std::fs::remove_file(&temp_overlay);
    let _ = std::fs::remove_file(&temp_merged);
    
    Ok(merged)
}

pub fn check_bun() -> bool {
    Command::new("bun")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
