use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub title: String,
    pub description: String,
    pub navbar_logo: Option<String>,
    pub favicon: Option<String>,
    pub theme: ThemeConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub primary_color: String,
    pub background_color: String,
    pub text_color: String,
    #[serde(alias = "admonitions")]
    pub colors: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        let default_yaml = include_str!("../resources/default_config.yml");
        serde_yaml::from_str(default_yaml).expect("Failed to parse default config")
    }
}
