use std::fs;
use std::path::PathBuf;
use anyhow::Result;

pub fn clean_project(path: &PathBuf, full: bool) -> Result<()> {
    let config_path = path.join("config.yml");
    if config_path.exists() {
        fs::remove_file(&config_path)?;
        println!("Removed {:?}", config_path);
    }

    let out_dir = path.join("out");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir)?;
        println!("Removed {:?}", out_dir);
    }

    let search_index_dir = path.join("search-index");
    if search_index_dir.exists() {
        fs::remove_dir_all(&search_index_dir)?;
        println!("Removed {:?}", search_index_dir);
    }

    // Find versioned directories
    let mut versioned_dirs = Vec::new();
    if path.exists() && path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    // Check if starts with 'v' followed by a digit
                    if name.starts_with('v') && name.chars().nth(1).map_or(false, |c| c.is_ascii_digit()) {
                        versioned_dirs.push(name.to_string());
                        if full {
                            fs::remove_dir_all(&p)?;
                            println!("Removed versioned directory: {:?}", name);
                        }
                    }
                }
            }
        }
    }

    if !full && !versioned_dirs.is_empty() {
        println!(
            "\nversioned directories ({}) not deleted. If you want to clear them as well, run 'verdocs clean --full'",
            versioned_dirs.join(", ")
        );
    }

    Ok(())
}
