use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use std::fs;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use pulldown_cmark::{Parser as MarkdownParser, Options, html, Event, Tag, TagEnd};
use fs_extra::dir::{copy, CopyOptions};
use crate::config::Config;
use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchIndexEntry {
    pub title: String,
    pub route: String,
    pub content: String,
}

pub struct VerdocsParser<'a> {
    config: &'a Config,
}

impl<'a> VerdocsParser<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn parse(&self, markdown: &str, version: u64) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);
        
        let parser = MarkdownParser::new_ext(markdown, options);
        let events: Vec<Event> = parser.collect();

        let events = self.apply_modifiers(events);

        let mut html_body = String::new();
        html::push_html(&mut html_body, events.into_iter());

        wrap_html(&html_body, &self.config.title, version)
    }

    fn apply_modifiers(&self, events: Vec<Event<'a>>) -> Vec<Event<'a>> {
        self.modifier_tags(events)
    }

    fn modifier_tags(&self, events: Vec<Event<'a>>) -> Vec<Event<'a>> {
        let mut new_events = Vec::new();
        let re_start = Regex::new(r#"(?s)\{([A-Z]+)(?:\s+type="([^"]*)")?(?:\s+title="([^"]*)")?\}"#).unwrap();
        let re_end = Regex::new(r#"\{/[A-Z]+\}"#).unwrap();

        // Use a stack to track whether we should close with </div> or </span>
        let mut tag_stack = Vec::new();

        let mut i = 0;
        while i < events.len() {
            let event = &events[i];

            match event {
                Event::Start(Tag::Paragraph) => {
                    // Peek ahead to see if this is a "tag-only" paragraph (Start -> Text -> End)
                    if let Some(Event::Text(text)) = events.get(i + 1) {
                        if let Some(Event::End(TagEnd::Paragraph)) = events.get(i + 2) {
                            let trimmed = text.trim();
                            
                            // Check for Standalone Start Tag
                            if let Some(caps) = re_start.captures(trimmed) {
                                if caps[0].len() == trimmed.len() {
                                    let tag_name = caps[1].to_lowercase();
                                    let tag_type = caps.get(2).map(|m| m.as_str());
                                    let title = caps.get(3).map(|m| m.as_str());

                                    if let Some(color) = self.config.theme.colors.get(&tag_name) {
                                        if tag_type == Some("admonition") {
                                            new_events.push(Event::Html(self.render_admonition_start(&tag_name, title, color).into()));
                                            tag_stack.push("div");
                                            i += 3;
                                            continue;
                                        }
                                    }
                                }
                            }
                            
                            // Check for Standalone End Tag
                            if re_end.is_match(trimmed) && trimmed.len() == re_end.find(trimmed).unwrap().as_str().len() {
                                let close_tag = tag_stack.pop().unwrap_or("div");
                                new_events.push(Event::Html(format!("</{}>", close_tag).into()));
                                i += 3;
                                continue;
                            }
                        }
                    }
                    new_events.push(event.clone());
                }
                Event::Text(text) => {
                    let mut final_content = text.to_string();
                    let mut has_tags = false;

                    // 1. Process Start Tags
                    // Note: This naive replacement doesn't handle nested tags within the SAME text block perfectly,
                    // but it works for standard usage.
                    while let Some(caps) = re_start.captures(&final_content) {
                        has_tags = true;
                        let full_match = caps[0].to_string();
                        let tag_name = caps[1].to_lowercase();
                        let tag_type = caps.get(2).map(|m| m.as_str());
                        let title = caps.get(3).map(|m| m.as_str());

                        if let Some(color) = self.config.theme.colors.get(&tag_name) {
                            let (html, close_type) = if tag_type == Some("admonition") {
                                (self.render_admonition_start(&tag_name, title, color), "div")
                            } else {
                                (format!(r#"<span style="color: {}; font-weight: bold;">"#, color), "span")
                            };
                            tag_stack.push(close_type);
                            final_content = final_content.replace(&full_match, &html);
                        } else {
                            break;
                        }
                    }

                    // 2. Process End Tags
                    while let Some(mat) = re_end.find(&final_content) {
                        has_tags = true;
                        let full_match = mat.as_str();
                        let close_tag = tag_stack.pop().unwrap_or("span");
                        final_content = final_content.replace(full_match, &format!("</{}>", close_tag));
                    }

                    if has_tags {
                        new_events.push(Event::Html(final_content.into()));
                    } else {
                        new_events.push(event.clone());
                    }
                }
                _ => {
                    new_events.push(event.clone());
                }
            }
            i += 1;
        }

        // Close any dangling tags
        while let Some(close_tag) = tag_stack.pop() {
            new_events.push(Event::Html(format!("</{}>", close_tag).into()));
        }

        new_events
    }

    fn render_admonition_start(&self, _tag_name: &str, title: Option<&str>, color: &str) -> String {
        let bg_color = hex_to_rgba(color, 0.1);
        let title_html = if let Some(t) = title {
            format!(r#"<div style="font-weight: bold; color: {}; margin-bottom: 5px;">{}</div>"#, color, t)
        } else {
            "".to_string()
        };

        format!(
            r#"<div class="admonition" style="padding: 10px 15px; margin-bottom: 15px; border-left: 5px solid {}; background-color: {}; border-radius: 6px; color: inherit;">{}"#,
            color, bg_color, title_html
        )
    }
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

    let assets_src = src_path.join("assets");
    if assets_src.exists() {
        let mut options = CopyOptions::new();
        options.copy_inside = true;
        copy(&assets_src, &out_dir, &options)?;
    }

    let verdocs_parser = VerdocsParser::new(&config);

    for entry in fs::read_dir(src_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            if dir_name.starts_with('v') {
                process_version(&path, &out_dir.join(dir_name), dir_name, &verdocs_parser, version)?;
            }
        }
    }

    Ok(())
}

fn process_version(version_src: &Path, version_out: &Path, version_name: &str, parser: &VerdocsParser, version: u64) -> Result<()> {
    fs::create_dir_all(version_out)?;
    let mut search_index = Vec::new();

    for entry in WalkDir::new(version_src) {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "md") {
            let content = fs::read_to_string(path)?;
            let full_html = parser.parse(&content, version);

            let relative_path = path.strip_prefix(version_src)?;
            let mut html_path = version_out.join(relative_path);
            html_path.set_extension("html");

            if let Some(parent) = html_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&html_path, &full_html)?;

            let route = relative_path.with_extension("").to_str().unwrap().to_string();
            search_index.push(SearchIndexEntry {
                title: route.clone(),
                route,
                content: content.chars().take(200).collect(),
            });
        }
    }

    let search_index_dir = version_out.parent().unwrap().join("search-index");
    fs::create_dir_all(&search_index_dir)?;
    let search_index_json = serde_json::to_string_pretty(&search_index)?;
    fs::write(search_index_dir.join(format!("{}.json", version_name.replace('.', "-"))), search_index_json)?;

    Ok(())
}

fn hex_to_rgba(hex: &str, opacity: f32) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return format!("rgba(0,0,0,{})", opacity);
    }
    
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    
    format!("rgba({}, {}, {}, {})", r, g, b, opacity)
}

fn wrap_html(body: &str, title: &str, version: u64) -> String {
    format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; padding: 2rem; max-width: 800px; margin: 0 auto; line-height: 1.6; color: #333; }}
        h1, h2, h3, h4, h5, h6 {{ color: #222; margin-top: 1.5em; }}
        code {{ background: #f4f4f4; padding: 0.2em 0.4em; border-radius: 3px; font-family: monospace; }}
        pre {{ background: #f4f4f4; padding: 1em; border-radius: 5px; overflow-x: auto; }}
        .admonition > *:first-child {{ margin-top: 0; }}
        .admonition > *:last-child {{ margin-bottom: 0; }}
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
