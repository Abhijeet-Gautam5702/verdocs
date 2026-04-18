use std::fs;
use std::path::PathBuf;
use anyhow::Result;

pub fn clean_project(path: &PathBuf) -> Result<()> {
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

    Ok(())
}
