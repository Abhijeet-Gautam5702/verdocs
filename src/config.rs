use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub title: String,
    pub description: String,
    pub base_path: Option<String>,
    pub navbar_logo: Option<String>,
    pub favicon: Option<String>,
    pub theme: ThemeConfig,
    pub dark_theme: ThemeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    pub primary_color: String,
    pub background_color: String,
    pub text_color: String,
    #[serde(alias = "admonitions")]
    pub colors: HashMap<String, String>,
}

impl Config {
    pub fn load(src_path: &Path) -> Result<Self> {
        let config_path = src_path.join("config.yml");
        let config_content = fs::read_to_string(&config_path)
            .with_context(|| format!("Could not read config file at {:?}", config_path))?;
        let config: Config = serde_yaml::from_str(&config_content)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        let default_yaml = include_str!("../resources/default_config.yml");
        serde_yaml::from_str(default_yaml).expect("Failed to parse default config")
    }
}
