use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use std::fs;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use pulldown_cmark::{Parser as MarkdownParser, Options, html};
use fs_extra::dir::{copy, CopyOptions};
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchIndexEntry {
    pub title: String,
    pub route: String,
    pub content: String,
}

pub fn generate_site(src_path: &PathBuf, version: u64) -> Result<()> {
    let config_path = src_path.join("config.yml");
    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("Could not read config file at {:?}", config_path))?;
    let config: Config = serde_yaml::from_str(&config_content)?;

    let out_dir = src_path.join("out");
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir)?;
    }
    fs::create_dir_all(&out_dir)?;

    // Copy assets if they exist
    let assets_src = src_path.join("assets");
    if assets_src.exists() {
        let mut options = CopyOptions::new();
        options.copy_inside = true;
        copy(&assets_src, &out_dir, &options)?;
    }

    // Iterate through versioned directories
    for entry in fs::read_dir(src_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            if dir_name.starts_with('v') {
                process_version(&path, &out_dir.join(dir_name), dir_name, &config, version)?;
            }
        }
    }

    Ok(())
}

fn process_version(version_src: &Path, version_out: &Path, version_name: &str, config: &Config, version: u64) -> Result<()> {
    fs::create_dir_all(version_out)?;
    let mut search_index = Vec::new();

    for entry in WalkDir::new(version_src) {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "md") {
            let content = fs::read_to_string(path)?;
            let html_body = markdown_to_html(&content);
            let full_html = wrap_html(&html_body, &config.title, version);

            let relative_path = path.strip_prefix(version_src)?;
            let mut html_path = version_out.join(relative_path);
            html_path.set_extension("html");

            if let Some(parent) = html_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&html_path, &full_html)?;

            let route = relative_path.with_extension("").to_str().unwrap().to_string();
            search_index.push(SearchIndexEntry {
                title: route.clone(), // For now, use route as title
                route,
                content: content.chars().take(200).collect(), // First 200 chars for preview
            });
        }
    }

    // Write search index for this version
    let search_index_dir = version_out.parent().unwrap().join("search-index");
    fs::create_dir_all(&search_index_dir)?;
    let search_index_json = serde_json::to_string_pretty(&search_index)?;
    fs::write(search_index_dir.join(format!("{}.json", version_name.replace('.', "-"))), search_index_json)?;

    Ok(())
}

fn wrap_html(body: &str, title: &str, version: u64) -> String {
    format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: sans-serif; padding: 2rem; max-width: 800px; margin: 0 auto; }}
    </style>
    <script>
        window.__verdocs_version = "{}";
        setInterval(() => {{
            fetch('/__verdocs/status')
                .then(r => r.text())
                .then(v => {{
                    if (v !== window.__verdocs_version) {{
                        location.reload();
                    }}
                }});
        }}, 1000);
    </script>
</head>
<body>
    {}
</body>
</html>"#, title, version, body)
}

fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = MarkdownParser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
