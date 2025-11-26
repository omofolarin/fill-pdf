use std::process::Command;
use std::io::{self, Write};

pub fn ensure_dependencies() -> anyhow::Result<()> {
    // Check Python3
    if !check_python3() {
        anyhow::bail!("Python 3 is not installed. Please install Python 3 first.");
    }
    
    // Check PyPDF2
    if !check_pypdf2() {
        println!("⚠️  PyPDF2 is not installed.");
        print!("Would you like to install it now? (y/N): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            install_pypdf2()?;
        } else {
            anyhow::bail!("PyPDF2 is required. Install with: pip3 install PyPDF2");
        }
    }
    
    Ok(())
}

fn check_python3() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_pypdf2() -> bool {
    Command::new("python3")
        .arg("-c")
        .arg("import PyPDF2")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn install_pypdf2() -> anyhow::Result<()> {
    println!("📦 Installing PyPDF2...");
    
    // Try pip3 first (most common)
    let pip_commands = ["pip3", "pip", "python3 -m pip", "python -m pip"];
    
    for pip_cmd in &pip_commands {
        let parts: Vec<&str> = pip_cmd.split_whitespace().collect();
        let (cmd, args) = if parts.len() > 1 {
            (parts[0], parts[1..].to_vec())
        } else {
            (parts[0], vec![])
        };
        
        let mut command = Command::new(cmd);
        for arg in args {
            command.arg(arg);
        }
        command.args(&["install", "PyPDF2"]);
        
        if let Ok(output) = command.output() {
            if output.status.success() {
                println!("✓ PyPDF2 installed successfully");
                return Ok(());
            }
        }
    }
    
    anyhow::bail!(
        "Failed to install PyPDF2. Please install manually:\n\
         - macOS/Linux: pip3 install PyPDF2\n\
         - Or: python3 -m pip install PyPDF2"
    )
}

pub fn merge_pdfs_bytes(template_bytes: &[u8], overlay_pdf: &[u8], flatten: bool) -> anyhow::Result<Vec<u8>> {
    let temp_dir = std::env::temp_dir();
    let temp_template = temp_dir.join("fill_pdf_template.pdf");
    let temp_overlay = temp_dir.join("fill_pdf_overlay.pdf");
    let temp_merged = temp_dir.join("fill_pdf_merged.pdf");
    
    std::fs::write(&temp_template, template_bytes)?;
    std::fs::write(&temp_overlay, overlay_pdf)?;
    
    let flatten_code = if flatten {
        r#"
    # Flatten form fields
    if '/AcroForm' in template.trailer['/Root']:
        del template.trailer['/Root']['/AcroForm']
    for page in template.pages:
        if '/Annots' in page:
            del page['/Annots']
"#
    } else {
        ""
    };
    
    let python_script = format!(r#"
import sys
try:
    from PyPDF2 import PdfReader, PdfWriter
    
    template = PdfReader('{}')
    overlay = PdfReader('{}')
    
    writer = PdfWriter()
    
    for i, page in enumerate(template.pages):
        if i < len(overlay.pages):
            page.merge_page(overlay.pages[i])
        writer.add_page(page)
    {}
    with open('{}', 'wb') as output:
        writer.write(output)
    
    print("SUCCESS")
    
except ImportError:
    print("ERROR: PyPDF2 not installed")
    sys.exit(1)
except Exception as e:
    print(f"ERROR: {{e}}")
    sys.exit(1)
"#, temp_template.display(), temp_overlay.display(), flatten_code, temp_merged.display());
    
    let output = Command::new("python3")
        .arg("-c")
        .arg(&python_script)
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("Merge failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    let merged = std::fs::read(&temp_merged)?;
    
    let _ = std::fs::remove_file(&temp_template);
    let _ = std::fs::remove_file(&temp_overlay);
    let _ = std::fs::remove_file(&temp_merged);
    
    Ok(merged)
}
