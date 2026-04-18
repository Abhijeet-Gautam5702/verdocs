use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use crate::config::Config;

pub fn init_project(path: &PathBuf) -> Result<()> {
    fs::create_dir_all(path.join("assets"))?;
    fs::create_dir_all(path.join("v1.0.0"))?;
    fs::create_dir_all(path.join("search-index"))?;

    let config = Config::default();
    let config_yaml = serde_yaml::to_string(&config)?;
    fs::write(path.join("config.yml"), config_yaml)?;

    let default_md = "# Home\n\nWelcome to your documentation site!\n";
    fs::write(path.join("v1.0.0").join("home.md"), default_md)?;

    Ok(())
}
