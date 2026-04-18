use std::path::{Path, PathBuf};
use anyhow::{Result, Context, anyhow};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarItem {
    pub title: String,
    pub route: String,
    pub children: Vec<SidebarItem>,
}

#[derive(Debug, Clone)]
pub struct TocItem {
    pub title: String,
    pub id: String,
    pub level: u32,
}

pub struct VerdocsParser<'a> {
    config: &'a Config,
    all_versions: Vec<String>,
}

impl<'a> VerdocsParser<'a> {
    pub fn new(config: &'a Config, all_versions: Vec<String>) -> Self {
        Self { config, all_versions }
    }

    pub fn parse(&self, markdown: &str, current_version: &str, current_route: &str, sidebar: &[SidebarItem], version_timestamp: u64) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);
        
        let parser = MarkdownParser::new_ext(markdown, options);
        let mut events: Vec<Event> = parser.collect();

        // 1. Extract TOC and inject IDs into headings
        let mut toc = Vec::new();
        let mut new_events = Vec::new();
        let mut i = 0;
        while i < events.len() {
            let event = &events[i];
            if let Event::Start(Tag::Heading { level, .. }) = event {
                let level_num = *level as u32;
                if level_num >= 2 {
                    let mut heading_text = String::new();
                    let mut j = i + 1;
                    while j < events.len() {
                        match &events[j] {
                            Event::Text(t) | Event::Code(t) => heading_text.push_str(t),
                            Event::End(TagEnd::Heading(_)) => break,
                            _ => {}
                        }
                        j += 1;
                    }
                    
                    let id = slugify(&heading_text);
                    toc.push(TocItem {
                        title: heading_text.clone(),
                        id: id.clone(),
                        level: level_num,
                    });

                    new_events.push(Event::Html(format!("<h{} id=\"{}\">", level_num, id).into()));
                    i += 1;
                    while i < events.len() {
                        if let Event::End(TagEnd::Heading(_)) = &events[i] {
                            new_events.push(Event::Html(format!("</h{}>", level_num).into()));
                            break;
                        }
                        new_events.push(events[i].clone());
                        i += 1;
                    }
                    i += 1;
                    continue;
                }
            }
            new_events.push(event.clone());
            i += 1;
        }
        events = new_events;

        // 2. Pass through modifiers
        let events = self.apply_modifiers(events, current_version, current_route);

        // 3. Compile AST to HTML
        let mut html_body = String::new();
        html::push_html(&mut html_body, events.into_iter());

        // 4. Wrap in full HTML template
        wrap_html(&html_body, &self.config.title, current_version, current_route, &self.all_versions, sidebar, &toc, version_timestamp)
    }

    fn apply_modifiers(&self, events: Vec<Event<'a>>, current_version: &str, current_route: &str) -> Vec<Event<'a>> {
        let events = self.modifier_tags(events);
        self.modifier_links(events, current_version, current_route)
    }

    fn modifier_links(&self, events: Vec<Event<'a>>, current_version: &str, current_route: &str) -> Vec<Event<'a>> {
        let mut new_events = Vec::new();
        let mut i = 0;
        while i < events.len() {
            match &events[i] {
                Event::Start(Tag::Link { link_type, dest_url, title, id }) => {
                    let dest = dest_url.to_string();
                    if dest.starts_with("http") {
                        // External link
                        new_events.push(Event::Html(format!(
                            r#"<a href="{}" title="{}" target="_blank" rel="noopener noreferrer">"#,
                            dest, title
                        ).into()));
                        
                        i += 1;
                        let mut depth = 1;
                        while i < events.len() && depth > 0 {
                            match &events[i] {
                                Event::Start(Tag::Link { .. }) => depth += 1,
                                Event::End(TagEnd::Link) => {
                                    depth -= 1;
                                    if depth == 0 {
                                        new_events.push(Event::Html("</a>".into()));
                                    }
                                }
                                _ => {
                                    if depth > 0 {
                                        new_events.push(events[i].clone());
                                    }
                                }
                            }
                            if depth > 0 { i += 1; }
                        }
                    } else {
                        // Internal link
                        let final_dest = resolve_internal_link(current_version, current_route, &dest);
                        new_events.push(Event::Start(Tag::Link {
                            link_type: *link_type,
                            dest_url: final_dest.into(),
                            title: title.clone(),
                            id: id.clone(),
                        }));
                    }
                }
                _ => {
                    new_events.push(events[i].clone());
                }
            }
            i += 1;
        }
        new_events
    }

    fn modifier_tags(&self, events: Vec<Event<'a>>) -> Vec<Event<'a>> {
        let mut new_events = Vec::new();
        let re_start = Regex::new(r#"(?s)\{([A-Z]+)(?:\s+type="([^"]*)")?(?:\s+title="([^"]*)")?\}"#).unwrap();
        let re_end = Regex::new(r#"\{/[A-Z]+\}"#).unwrap();

        let mut tag_stack = Vec::new();
        let mut in_code_block = false;

        let mut i = 0;
        while i < events.len() {
            let event = &events[i];

            match event {
                Event::Start(Tag::CodeBlock(_)) => {
                    in_code_block = true;
                    new_events.push(event.clone());
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    new_events.push(event.clone());
                }
                Event::Start(Tag::Paragraph) if !in_code_block => {
                    if let Some(Event::Text(text)) = events.get(i + 1) {
                        if let Some(Event::End(TagEnd::Paragraph)) = events.get(i + 2) {
                            let trimmed = text.trim();
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
                Event::Text(text) if !in_code_block => {
                    let mut final_content = text.to_string();
                    let mut has_tags = false;

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

fn resolve_internal_link(current_version: &str, current_route: &str, dest: &str) -> String {
    let mut path_to_resolve = dest.to_string();

    if path_to_resolve.starts_with("@/") {
        // Global path within current version: @/path/to/file.md -> /v1/path/to/file
        path_to_resolve = path_to_resolve[2..].to_string();
    } else if path_to_resolve.starts_with('/') {
        path_to_resolve = path_to_resolve.trim_start_matches('/').to_string();
    } else {
        // Resolve relative path
        let base_path = PathBuf::from(current_route);
        let mut final_path = base_path;
        for component in Path::new(&path_to_resolve).components() {
            match component {
                std::path::Component::ParentDir => {
                    final_path.pop();
                }
                std::path::Component::Normal(c) => {
                    final_path.push(c);
                }
                _ => {}
            }
        }
        path_to_resolve = final_path.to_str().unwrap_or(&path_to_resolve).to_string().replace("\\", "/");
    }

    // Strip .md and handle index collapse
    if path_to_resolve.ends_with(".md") {
        path_to_resolve = path_to_resolve[..path_to_resolve.len() - 3].to_string();
    }

    let parts: Vec<&str> = path_to_resolve.split('/').collect();
    if parts.len() >= 2 {
        let last = parts[parts.len() - 1];
        let second_last = parts[parts.len() - 2];
        if last == second_last {
            path_to_resolve = parts[..parts.len() - 1].join("/");
        }
    }

    format!("/{}/{}", current_version, path_to_resolve.trim_start_matches('/'))
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

pub fn generate_site(src_path: &PathBuf, version_timestamp: u64) -> Result<()> {
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

    let mut all_versions = Vec::new();
    for entry in fs::read_dir(src_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            if dir_name.starts_with('v') {
                all_versions.push(dir_name.to_string());
            }
        }
    }
    all_versions.sort();

    let verdocs_parser = VerdocsParser::new(&config, all_versions);

    for entry in fs::read_dir(src_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            if dir_name.starts_with('v') {
                process_version(&path, &out_dir.join(dir_name), dir_name, &verdocs_parser, version_timestamp)?;
            }
        }
    }

    Ok(())
}

fn process_version(version_src: &Path, version_out: &Path, version_name: &str, parser: &VerdocsParser, version_timestamp: u64) -> Result<()> {
    fs::create_dir_all(version_out)?;
    let mut search_index = Vec::new();

    for entry in WalkDir::new(version_src) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if path == version_src { continue; }
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            let index_md = path.join(format!("{}.md", dir_name));
            if !index_md.exists() {
                return Err(anyhow!("Missing index file: {:?}", index_md));
            }
        }
    }

    let sidebar = build_sidebar(version_src, version_src)?;

    for entry in WalkDir::new(version_src) {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "md") {
            let content = fs::read_to_string(path)?;
            let relative_path = path.strip_prefix(version_src)?;
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            
            let mut route_parts = Vec::new();
            for component in relative_path.components() {
                let name = component.as_os_str().to_str().unwrap();
                if name.ends_with(".md") {
                    let stem = &name[..name.len()-3];
                    if let Some(parent) = relative_path.parent() {
                        if let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) {
                            if stem == parent_name { continue; }
                        }
                    }
                    route_parts.push(stem);
                } else {
                    route_parts.push(name);
                }
            }
            
            let route = route_parts.join("/");
            let full_html = parser.parse(&content, version_name, &route, &sidebar, version_timestamp);
            
            let mut html_path = version_out.join(&route);
            if !route.is_empty() {
                fs::create_dir_all(&html_path)?;
                html_path = html_path.join("index.html");
            } else {
                html_path = html_path.join("index.html");
            }

            if let Some(parent) = html_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&html_path, &full_html)?;

            search_index.push(SearchIndexEntry {
                title: file_stem.to_string(),
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

fn build_sidebar(current_dir: &Path, root_dir: &Path) -> Result<Vec<SidebarItem>> {
    let mut items = Vec::new();
    let entries = fs::read_dir(current_dir)?;
    let mut paths: Vec<_> = entries.map(|e| e.unwrap().path()).collect();
    
    paths.sort_by(|a, b| {
        let a_name = a.file_name().unwrap().to_str().unwrap().to_lowercase();
        let b_name = b.file_name().unwrap().to_str().unwrap().to_lowercase();
        if a_name == "home" { return std::cmp::Ordering::Less; }
        if b_name == "home" { return std::cmp::Ordering::Greater; }
        a_name.cmp(&b_name)
    });

    for path in paths {
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            let relative_path = path.strip_prefix(root_dir)?;
            let route = relative_path.to_str().unwrap().replace("\\", "/");
            items.push(SidebarItem {
                title: to_title_case(dir_name),
                route,
                children: build_sidebar(&path, root_dir)?,
            });
        } else if path.extension().map_or(false, |ext| ext == "md") {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let parent_name = current_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if stem == parent_name { continue; }
            let relative_path = path.strip_prefix(root_dir)?;
            let route = relative_path.with_extension("").to_str().unwrap().replace("\\", "/");
            items.push(SidebarItem {
                title: to_title_case(stem),
                route,
                children: Vec::new(),
            });
        }
    }
    Ok(items)
}

fn to_title_case(s: &str) -> String {
    s.split('-').map(|word| {
        let mut c = word.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }).collect::<Vec<_>>().join(" ")
}

fn hex_to_rgba(hex: &str, opacity: f32) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return format!("rgba(0,0,0,{})", opacity); }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    format!("rgba({}, {}, {}, {})", r, g, b, opacity)
}

fn wrap_html(body: &str, title: &str, current_version: &str, current_route: &str, all_versions: &[String], sidebar: &[SidebarItem], toc: &[TocItem], version_timestamp: u64) -> String {
    let version_options: String = all_versions.iter().map(|v| {
        format!(r#"<option value="{}" {}>{}</option>"#, v, if v == current_version { "selected" } else { "" }, v)
    }).collect::<Vec<_>>().join("\n");

    let sidebar_html = render_sidebar(sidebar, current_version, current_route, 0);
    
    let has_toc = !toc.is_empty();
    let toc_html: String = if has_toc {
        toc.iter().map(|item| {
            let indent = (item.level - 2) * 15;
            format!(r##"<a href="#{}" class="toc-item" style="padding-left: {}px;">{}</a>"##, item.id, indent, item.title)
        }).collect::<Vec<_>>().join("\n")
    } else {
        String::new()
    };

    let main_content_style = if has_toc {
        "margin-right: var(--minimap-width);"
    } else {
        "margin-right: 0;"
    };

    let minimap_html = if has_toc {
        format!(r##"<div id="minimap">
        <div class="toc-title">On this page</div>
        {}
    </div>"##, toc_html)
    } else {
        String::new()
    };

    format!(r##"<!DOCTYPE html>
<html style="scroll-padding-top: 40px;">
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css">
    <style>
        :root {{
            --primary-color: #007bff;
            --bg-color: #ffffff;
            --text-color: #333333;
            --sidebar-width: 260px;
            --minimap-width: 240px;
        }}
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; 
            margin: 0;
            display: flex;
            background: var(--bg-color);
            color: var(--text-color);
        }}
        
        #sidebar {{
            width: var(--sidebar-width);
            height: 100vh;
            position: fixed;
            left: 0;
            top: 0;
            border-right: 1px solid #eee;
            background: #fcfcfc;
            overflow-y: auto;
            padding: 20px 0;
            box-sizing: border-box;
        }}

        #minimap {{
            width: var(--minimap-width);
            height: 100vh;
            position: fixed;
            right: 0;
            top: 0;
            border-left: 1px solid #eee;
            background: #fcfcfc;
            overflow-y: auto;
            padding: 60px 20px 20px 20px;
            box-sizing: border-box;
        }}

        #main-content {{
            margin-left: var(--sidebar-width);
            {}
            padding: 2rem 4rem 10rem 4rem;
            flex-grow: 1;
            margin-top: 20px;
            max-width: 1000px;
        }}

        .sidebar-item, .toc-item {{
            display: block;
            padding: 8px 20px;
            text-decoration: none;
            color: #555;
            font-size: 14px;
            transition: all 0.2s;
        }}
        .toc-item {{ padding: 4px 0; font-size: 13px; color: #777; }}
        .sidebar-item:hover, .toc-item:hover {{ color: var(--primary-color); }}
        .sidebar-item.active {{
            background: #eef6ff;
            color: var(--primary-color);
            font-weight: bold;
            border-right: 3px solid var(--primary-color);
        }}

        .sidebar-group {{ display: none; }}
        .sidebar-group.expanded {{ display: block; }}

        h1, h2, h3, h4, h5, h6 {{ color: #222; margin-top: 1.5em; scroll-margin-top: 40px; }}
        code {{ 
            background: #f1f1f1; 
            padding: 0.2em 0.4em; 
            border-radius: 3px; 
            font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace;
            font-size: 85%;
        }}
        pre {{ 
            background: #0d1117; 
            padding: 32px 16px 12px 16px; 
            border-radius: 8px; 
            overflow-x: auto; 
            line-height: 1.45;
            border: 1px solid #30363d;
            position: relative;
            margin: 1.5rem 0;
        }}
        pre::before {{
            content: attr(data-lang);
            position: absolute;
            top: 10px;
            left: 16px;
            font-size: 11px;
            font-weight: bold;
            color: #8b949e;
            text-transform: uppercase;
            font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace;
        }}
        pre code {{
            background: transparent !important;
            padding: 0 !important;
            color: #e6edf3;
            font-size: 13px;
            display: block;
        }}
        .copy-button {{
            position: absolute;
            top: 10px;
            right: 10px;
            padding: 4px 8px;
            background: rgba(255,255,255,0.05);
            color: #8b949e;
            border: 1px solid rgba(255,255,255,0.1);
            border-radius: 6px;
            cursor: pointer;
            font-size: 11px;
            font-weight: bold;
            transition: all 0.2s;
            opacity: 0;
        }}
        pre:hover .copy-button {{
            opacity: 1;
        }}
        .copy-button:hover {{
            background: rgba(255,255,255,0.1);
            color: #fff;
            border-color: rgba(255,255,255,0.2);
        }}
        .admonition > *:first-child {{ margin-top: 0; }}
        .admonition > *:last-child {{ margin-bottom: 0; }}
        
        #version-selector-container {{
            padding: 10px 20px;
            margin-bottom: 10px;
            border-bottom: 1px solid #eee;
        }}
        #version-selector-container select {{
            width: 100%;
            padding: 5px;
            border: 1px solid #ddd;
            border-radius: 4px;
            background: #fff;
            outline: none;
        }}
        .toc-title {{
            font-size: 11px;
            text-transform: uppercase;
            letter-spacing: 1px;
            color: #999;
            margin-bottom: 10px;
            font-weight: bold;
        }}
    </style>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
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

        function switchVersion(newVersion) {{
            const currentPath = window.location.pathname;
            const pathParts = currentPath.split('/');
            const versionIndex = pathParts.findIndex(part => part === "{}");
            if (versionIndex !== -1) {{
                pathParts[versionIndex] = newVersion;
                window.location.pathname = pathParts.join('/');
            }} else {{
                window.location.pathname = '/' + newVersion + '/home';
            }}
        }}

        document.addEventListener('DOMContentLoaded', () => {{
            // 1. Extract and set language labels
            document.querySelectorAll('pre code').forEach(code => {{
                const pre = code.parentElement;
                const langClass = Array.from(code.classList).find(c => c.startsWith('language-'));
                if (langClass) {{
                    const lang = langClass.replace('language-', '');
                    pre.setAttribute('data-lang', lang);
                }}
            }});

            // 2. Highlight Code
            hljs.highlightAll();

            // 3. Add Copy Buttons
            document.querySelectorAll('pre').forEach(pre => {{
                const button = document.createElement('button');
                button.innerText = 'Copy';
                button.className = 'copy-button';
                pre.appendChild(button);
                
                button.onclick = () => {{
                    const code = pre.querySelector('code').innerText;
                    navigator.clipboard.writeText(code).then(() => {{
                        button.innerText = 'Copied!';
                        setTimeout(() => {{
                            button.innerText = 'Copy';
                        }}, 2000);
                    }});
                }};
            }});
        }});
    </script>
</head>
<body>
    <div id="sidebar">
        <div id="version-selector-container">
            <select onchange="switchVersion(this.value)">
                {}
            </select>
        </div>
        {}
    </div>
    <div id="main-content">
        {}
    </div>
    {}
</body>
</html>"##, title, main_content_style, version_timestamp, current_version, version_options, sidebar_html, body, minimap_html)
}

fn render_sidebar(items: &[SidebarItem], version: &str, current_route: &str, depth: usize) -> String {
    let mut html = String::new();
    let indent = depth * 15 + 20;
    for item in items {
        let is_active = item.route == current_route;
        let is_descendant = current_route.starts_with(&item.route) && !item.route.is_empty();
        let active_class = if is_active { "active" } else { "" };
        html.push_str(&format!(
            r#"<a href="/{}/{}" class="sidebar-item {}" style="padding-left: {}px;">{}</a>"#,
            version, item.route, active_class, indent, item.title
        ));
        if !item.children.is_empty() {
            let expanded_class = if is_descendant || is_active { "expanded" } else { "" };
            html.push_str(&format!(r#"<div class="sidebar-group {}">"#, expanded_class));
            html.push_str(&render_sidebar(&item.children, version, current_route, depth + 1));
            html.push_str("</div>");
        }
    }
    html
}
