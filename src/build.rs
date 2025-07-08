#![allow(unused_imports)]
use std::fs::{ self, File };
use std::io::{ BufRead, BufReader };
use std::path::{ Path, PathBuf };
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=ui/assets/styles/scss");
    
    // compile SCSS:
    compile_scss_dir("ui/assets/styles/scss", "ui/assets/styles/css")?;
    
    // build tauri:
    tauri_build::build();

    Ok(())
}

/// Compiles SCSS files dir
fn compile_scss_dir<P: AsRef<Path>>(input_dir: P, output_dir: P) -> Result<(), Box<dyn std::error::Error>> {
    let input_dir = input_dir.as_ref();
    let output_dir = output_dir.as_ref();
    
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();

        // recursive dirs:
        if path.is_dir() {
            // skip dirs named 'modules':
            if path.file_name().map_or(false, |name| name == "modules") { continue; }
            
            // prepare output path:
            let rel = path.strip_prefix(input_dir)?;
            let out_subdir = output_dir.join(rel);

            compile_scss_dir(&path, &out_subdir)?;
        }
        
        // compile scss files:
        else if path.extension().map_or(false, |ext| ext == "scss") {
            // skip file with comment in first line:
            let mut reader = BufReader::new(File::open(&path)?);
            let mut first_line = String::new();

            reader.read_line(&mut first_line)?;
            let trimmed = first_line.trim_start();
            
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            // prepare output path:
            let rel = path.strip_prefix(input_dir)?.with_extension("css");
            let out_path = output_dir.join(rel);

            compile_scss_file(path, out_path)?;
        }
    }

    Ok(())
}

/// Compiles SCSS file
fn compile_scss_file<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<(), Box<dyn std::error::Error>> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    // compile into CSS:
    let css = grass::from_path(&input_path, &grass::Options::default())?;
    
    // write results to file:
    fs::create_dir_all(output_path.parent().unwrap())?;
    fs::write(output_path, &css)?;
    
    Ok(())
}
