use std::process::Command;
use std::io::{self, Write};

pub fn ensure_dependencies(backend: &str) -> anyhow::Result<()> {
    match backend {
        "python" => ensure_python_deps(),
        "bun" => ensure_bun_deps(),
        _ => anyhow::bail!("Unknown backend: {}. Use 'python' or 'bun'", backend),
    }
}

fn ensure_python_deps() -> anyhow::Result<()> {
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

fn ensure_bun_deps() -> anyhow::Result<()> {
    // Check Bun
    if !check_bun() {
        anyhow::bail!("Bun is not installed. Install from: https://bun.sh");
    }
    println!("✓ Bun runtime available");
    
    // Check pdf-lib
    if !check_pdf_lib() {
        println!("⚠️  pdf-lib is not installed.");
        print!("Would you like to install it now? (y/N): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            install_pdf_lib()?;
        } else {
            anyhow::bail!("pdf-lib is required. Install with: bun install pdf-lib");
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

pub fn merge_pdfs_bytes(template_bytes: &[u8], overlay_pdf: &[u8], flatten: bool, backend: &str) -> anyhow::Result<Vec<u8>> {
    let start = std::time::Instant::now();
    
    let result = if backend == "bun" {
        merge_with_bun(template_bytes, overlay_pdf, flatten)?
    } else {
        merge_with_python(template_bytes, overlay_pdf, flatten)?
    };
    
    let duration = start.elapsed();
    println!("⏱️  Merge completed in {:.2}ms using {}", duration.as_secs_f64() * 1000.0, backend);
    
    Ok(result)
}

fn merge_with_python(template_bytes: &[u8], overlay_pdf: &[u8], flatten: bool) -> anyhow::Result<Vec<u8>> {
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

fn merge_with_bun(template_bytes: &[u8], overlay_pdf: &[u8], flatten: bool) -> anyhow::Result<Vec<u8>> {
    let temp_dir = std::env::temp_dir();
    let temp_template = temp_dir.join("fill_pdf_template_bun.pdf");
    let temp_overlay = temp_dir.join("fill_pdf_overlay_bun.pdf");
    let temp_merged = temp_dir.join("fill_pdf_merged_bun.pdf");
    
    std::fs::write(&temp_template, template_bytes)?;
    std::fs::write(&temp_overlay, overlay_pdf)?;
    
    let script_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("merge_pdfs.ts");
    
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
    
    let merged = std::fs::read(&temp_merged)?;
    
    let _ = std::fs::remove_file(&temp_template);
    let _ = std::fs::remove_file(&temp_overlay);
    let _ = std::fs::remove_file(&temp_merged);
    
    Ok(merged)
}

fn check_bun() -> bool {
    Command::new("bun")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn check_pdf_lib() -> bool {
    Command::new("bun")
        .arg("pm")
        .arg("ls")
        .output()
        .map(|o| {
            o.status.success() && 
            String::from_utf8_lossy(&o.stdout).contains("pdf-lib")
        })
        .unwrap_or(false)
}

fn install_pdf_lib() -> anyhow::Result<()> {
    println!("📦 Installing pdf-lib...");
    
    let output = Command::new("bun")
        .arg("install")
        .arg("pdf-lib")
        .output()?;
    
    if output.status.success() {
        println!("✓ pdf-lib installed successfully");
        Ok(())
    } else {
        anyhow::bail!(
            "Failed to install pdf-lib. Please install manually:\n\
             bun install pdf-lib"
        )
    }
}
