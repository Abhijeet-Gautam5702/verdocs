use crate::cli::Host;
use crate::config::Config;
use anyhow::{anyhow, Context, Result};
use fs_extra::dir::{copy, CopyOptions};
use pulldown_cmark::{html, Event, Options, Parser as MarkdownParser, Tag, TagEnd};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
    logo_exists: bool,
}

impl<'a> VerdocsParser<'a> {
    pub fn new(config: &'a Config, all_versions: Vec<String>, logo_exists: bool) -> Self {
        Self {
            config,
            all_versions,
            logo_exists,
        }
    }

    pub fn parse(
        &self,
        markdown: &str,
        current_version: &str,
        current_route: &str,
        sidebar: &[SidebarItem],
        version_timestamp: u64,
    ) -> (String, Vec<TocItem>, String) {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);

        let parser = MarkdownParser::new_ext(markdown, options);
        let mut events: Vec<Event> = parser.collect();

        // 1. Extract TOC and H1 Title
        let mut toc = Vec::new();
        let mut h1_title = String::new();
        let mut new_events = Vec::new();
        let mut i = 0;
        while i < events.len() {
            let event = &events[i];
            if let Event::Start(Tag::Heading { level, .. }) = event {
                let level_num = *level as u32;

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

                if level_num == 1 && h1_title.is_empty() {
                    h1_title = heading_text.clone();
                }

                if level_num >= 2 {
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
        let full_html = wrap_html(
            &html_body,
            self.config,
            current_version,
            current_route,
            &self.all_versions,
            sidebar,
            &toc,
            version_timestamp,
            self.logo_exists,
        );

        (full_html, toc, h1_title)
    }

    fn apply_modifiers(
        &self,
        events: Vec<Event<'a>>,
        current_version: &str,
        current_route: &str,
    ) -> Vec<Event<'a>> {
        let events = self.modifier_tags(events);
        let events = self.modifier_links(events, current_version, current_route);
        self.modifier_images(events)
    }

    fn modifier_images(&self, events: Vec<Event<'a>>) -> Vec<Event<'a>> {
        let mut new_events = Vec::new();
        for event in events {
            if let Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) = event
            {
                let dest = dest_url.to_string();
                let base_path = self.config.base_path.clone().unwrap_or_default();
                let base_path_url = if base_path.is_empty() {
                    "".to_string()
                } else {
                    format!("/{}", base_path.trim_matches('/'))
                };

                let final_dest = if dest.starts_with("assets/") {
                    format!("{}/{}", base_path_url, dest)
                } else {
                    dest
                };
                new_events.push(Event::Start(Tag::Image {
                    link_type,
                    dest_url: final_dest.into(),
                    title,
                    id,
                }));
            } else {
                new_events.push(event);
            }
        }
        new_events
    }

    fn modifier_links(
        &self,
        events: Vec<Event<'a>>,
        current_version: &str,
        current_route: &str,
    ) -> Vec<Event<'a>> {
        let mut new_events = Vec::new();
        let mut i = 0;
        while i < events.len() {
            match &events[i] {
                Event::Start(Tag::Link {
                    link_type,
                    dest_url,
                    title,
                    id,
                }) => {
                    let dest = dest_url.to_string();
                    if dest.starts_with("http") {
                        new_events.push(Event::Html(
                            format!(
                                r#"<a href="{}" title="{}" target="_blank" rel="noopener noreferrer">"#,
                                dest, title
                            )
                            .into(),
                        ));

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
                            if depth > 0 {
                                i += 1;
                            }
                        }
                    } else {
                        let base_path = self.config.base_path.clone().unwrap_or_default();
                        let final_dest =
                            resolve_internal_link(current_version, current_route, &dest, &base_path);
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
        let re_start =
            Regex::new(r#"(?s)\{([A-Z]+)(?:\s+type="([^"]*)")?(?:\s+title="([^"]*)")?\}"#).unwrap();
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

                                    if let Some(_color) = self.config.theme.colors.get(&tag_name) {
                                        if tag_type == Some("admonition") {
                                            new_events.push(Event::Html(
                                                self.render_admonition_start(&tag_name, title)
                                                    .into(),
                                            ));
                                            tag_stack.push("div");
                                            i += 3;
                                            continue;
                                        }
                                    }
                                }
                            }
                            if re_end.is_match(trimmed)
                                && trimmed.len() == re_end.find(trimmed).unwrap().as_str().len()
                            {
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

                        if let Some(_color) = self.config.theme.colors.get(&tag_name) {
                            let (html, close_type) = if tag_type == Some("admonition") {
                                (self.render_admonition_start(&tag_name, title), "div")
                            } else {
                                (
                                    format!(
                                        r#"<span style="color: var(--color-{}); font-weight: bold;">"#,
                                        tag_name
                                    ),
                                    "span",
                                )
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
                        final_content =
                            final_content.replace(full_match, &format!("</{}>", close_tag));
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

    fn render_admonition_start(&self, tag_name: &str, title: Option<&str>) -> String {
        let title_html = if let Some(t) = title {
            format!(
                r#"<div style="font-weight: bold; color: var(--color-{}); margin-bottom: 5px;">{}</div>"#,
                tag_name, t
            )
        } else {
            "".to_string()
        };

        format!(
            r#"<div class="admonition" style="padding: 10px 15px; margin-bottom: 15px; border-left: 5px solid var(--color-{}); background-color: var(--bg-color-{}); border-radius: 6px; color: inherit;">{}"#,
            tag_name, tag_name, title_html
        )
    }
}

fn resolve_internal_link(
    current_version: &str,
    current_route: &str,
    dest: &str,
    base_path: &str,
) -> String {
    let mut path_to_resolve = dest.to_string();

    if path_to_resolve.starts_with("@/") {
        path_to_resolve = path_to_resolve[2..].to_string();
    } else if path_to_resolve.starts_with('/') {
        path_to_resolve = path_to_resolve.trim_start_matches('/').to_string();
    } else {
        let bp = PathBuf::from(current_route);
        let mut final_path = bp;
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
        path_to_resolve = final_path
            .to_str()
            .unwrap_or(&path_to_resolve)
            .to_string()
            .replace("\\", "/");
    }

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

    let base = if base_path.is_empty() {
        "".to_string()
    } else {
        format!("/{}", base_path.trim_matches('/'))
    };

    format!(
        "{}/{}/{}",
        base,
        current_version,
        path_to_resolve.trim_start_matches('/')
    )
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

pub fn generate_site(src_path: &PathBuf, version_timestamp: u64, host: Host) -> Result<()> {
    let config_path = src_path.join("config.yml");
    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("Could not read config file at {:?}", config_path))?;
    let config: Config = serde_yaml::from_str(&config_content)?;

    let mut logo_exists = false;
    if let Some(ref logo) = config.navbar_logo {
        let logo_path = src_path.join("assets").join(logo);
        if logo_path.exists() {
            logo_exists = true;
        } else {
            println!(
                "Warning: Navbar logo not found at: {:?}. Using title instead.",
                logo_path
            );
        }
    } else {
        println!("Note: No navbar_logo provided in config.yml. Using title instead.");
    }

    if let Some(ref favicon) = config.favicon {
        let favicon_path = src_path.join("assets").join(favicon);
        if !favicon_path.exists() {
            return Err(anyhow!("Favicon not found at: {:?}", favicon_path));
        }
    }

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

    let verdocs_parser = VerdocsParser::new(&config, all_versions.clone(), logo_exists);

    for entry in fs::read_dir(src_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            if dir_name.starts_with('v') {
                process_version(
                    &path,
                    &out_dir.join(dir_name),
                    dir_name,
                    &verdocs_parser,
                    version_timestamp,
                )?;
            }
        }
    }

    // Create root index.html redirecting to the latest version
    if let Some(latest_version) = all_versions.last() {
        let base_path = config.base_path.clone().unwrap_or_default();
        let base_path_url = if base_path.is_empty() {
            "".to_string()
        } else {
            format!("/{}", base_path.trim_matches('/'))
        };

        let redirect_url = format!("{}/{}/home", base_path_url, latest_version);
        let redirect_html = format!(
            r##"<!DOCTYPE html>
<html>
<head>
    <meta http-equiv="refresh" content="0; url={}" />
    <script type="text/javascript">
        window.location.href = "{}"
    </script>
</head>
<body>
    <p>If you are not redirected, <a href="{}">click here</a>.</p>
</body>
</html>"##,
            redirect_url, redirect_url, redirect_url
        );
        fs::write(out_dir.join("index.html"), redirect_html)?;
    }

    // Host-specific optimizations
    match host {
        Host::Vercel => {
            let vercel_json = r#"{
  "cleanUrls": true
}"#;
            fs::write(out_dir.join("vercel.json"), vercel_json)?;
        }
        Host::GhPages => {
            fs::write(out_dir.join(".nojekyll"), "")?;
        }
        Host::Vps => {}
    }

    Ok(())
}

fn process_version(
    version_src: &Path,
    version_out: &Path,
    version_name: &str,
    parser: &VerdocsParser,
    version_timestamp: u64,
) -> Result<()> {
    fs::create_dir_all(version_out)?;
    let mut search_index = Vec::new();

    for entry in WalkDir::new(version_src) {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if path == version_src {
                continue;
            }
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
                    let stem = &name[..name.len() - 3];
                    if let Some(parent) = relative_path.parent() {
                        if let Some(parent_name) = parent.file_name().and_then(|n| n.to_str()) {
                            if stem == parent_name {
                                continue;
                            }
                        }
                    }
                    route_parts.push(stem);
                } else {
                    route_parts.push(name);
                }
            }

            let route = route_parts.join("/");
            let (full_html, _toc, h1_title) =
                parser.parse(&content, version_name, &route, &sidebar, version_timestamp);

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
                title: if h1_title.is_empty() {
                    file_stem.to_string()
                } else {
                    h1_title
                },
                route,
                content: content.clone(),
            });
        }
    }

    let search_index_dir = version_out.parent().unwrap().join("search-index");
    fs::create_dir_all(&search_index_dir)?;
    let search_index_json = serde_json::to_string_pretty(&search_index)?;
    fs::write(
        search_index_dir.join(format!("{}.json", version_name.replace('.', "-"))),
        search_index_json,
    )?;

    Ok(())
}

fn build_sidebar(current_dir: &Path, root_dir: &Path) -> Result<Vec<SidebarItem>> {
    let mut items = Vec::new();
    let entries = fs::read_dir(current_dir)?;
    let mut paths: Vec<_> = entries.map(|e| e.unwrap().path()).collect();

    paths.sort_by(|a, b| {
        let a_name = a.file_name().unwrap().to_str().unwrap().to_lowercase();
        let b_name = b.file_name().unwrap().to_str().unwrap().to_lowercase();
        if a_name == "home" {
            return std::cmp::Ordering::Less;
        }
        if b_name == "home" {
            return std::cmp::Ordering::Greater;
        }
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
            let parent_name = current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if stem == parent_name {
                continue;
            }
            let relative_path = path.strip_prefix(root_dir)?;
            let route = relative_path
                .with_extension("")
                .to_str()
                .unwrap()
                .replace("\\", "/");
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
    s.split('-')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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

fn wrap_html(
    body: &str,
    config: &Config,
    current_version: &str,
    current_route: &str,
    all_versions: &[String],
    sidebar: &[SidebarItem],
    toc: &[TocItem],
    version_timestamp: u64,
    logo_exists: bool,
) -> String {
    let base_path = config.base_path.clone().unwrap_or_default();
    let base_path_url = if base_path.is_empty() {
        "".to_string()
    } else {
        format!("/{}", base_path.trim_matches('/'))
    };

    let version_options: String = all_versions
        .iter()
        .map(|v| {
            format!(
                r#"<option value="{}" {}>{}</option>"#,
                v,
                if v == current_version { "selected" } else { "" },
                v
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let sidebar_html = render_sidebar(sidebar, current_version, current_route, 0, &base_path_url);

    let has_toc = !toc.is_empty();
    let toc_html: String = if has_toc {
        toc.iter()
            .map(|item| {
                let indent = (item.level - 2) * 15;
                format!(
                    r##"<a href="#{}" class="toc-item" style="padding-left: {}px;">{}</a>"##,
                    item.id, indent, item.title
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };

    let main_content_style = if has_toc {
        "margin-right: var(--minimap-width);"
    } else {
        "margin-right: 0;"
    };

    let minimap_html = if has_toc {
        format!(
            r##"<div id="minimap">
        <div class="toc-title">On this page</div>
        {}
    </div>"##,
            toc_html
        )
    } else {
        String::new()
    };

    let mut light_vars = format!(
        "--primary-color: {}; --bg-color: {}; --text-color: {}; --navbar-bg: #ffffff; --sidebar-bg: #fcfcfc; --border-color: #eeeeee; --code-bg: #f1f1f1; --search-bg: #f3f4f6; --search-border: #e5e7eb; --search-text: #6b7280; --h-color: #222; --sidebar-item-color: #555; --toc-item-color: #777; --modal-bg: #ffffff;",
        config.theme.primary_color, config.theme.background_color, config.theme.text_color
    );
    for (name, color) in &config.theme.colors {
        light_vars.push_str(&format!("--color-{}: {};", name, color));
        light_vars.push_str(&format!("--bg-color-{}: {};", name, hex_to_rgba(color, 0.1)));
    }

    let mut dark_vars = format!(
        "--primary-color: {}; --bg-color: {}; --text-color: {}; --navbar-bg: #161b22; --sidebar-bg: #0d1117; --border-color: #30363d; --code-bg: #21262d; --search-bg: #21262d; --search-border: #30363d; --search-text: #8b949e; --h-color: #e6edf3; --sidebar-item-color: #8b949e; --toc-item-color: #7d8590; --modal-bg: #161b22;",
        config.dark_theme.primary_color, config.dark_theme.background_color, config.dark_theme.text_color
    );
    for (name, color) in &config.dark_theme.colors {
        dark_vars.push_str(&format!("--color-{}: {};", name, color));
        dark_vars.push_str(&format!("--bg-color-{}: {};", name, hex_to_rgba(color, 0.1)));
    }

    let navbar_logo_html = if logo_exists {
        if let Some(ref logo) = config.navbar_logo {
            format!(
                r#"<img src="{}/assets/{}" alt="Logo" style="height: 32px; cursor: pointer;" onclick="window.location.href='{}/{}/home'">"#,
                base_path_url, logo, base_path_url, current_version
            )
        } else {
            // This case should theoretically not be hit if logo_exists is true, but for safety:
            format!(
                r#"<div style="font-size: 20px; font-weight: bold; color: var(--primary-color); cursor: pointer;" onclick="window.location.href='{}/{}/home'">{}</div>"#,
                base_path_url, current_version, config.title
            )
        }
    } else {
        format!(
            r#"<div style="font-size: 20px; font-weight: bold; color: var(--primary-color); cursor: pointer;" onclick="window.location.href='{}/{}/home'">{}</div>"#,
            base_path_url, current_version, config.title
        )
    };

    let favicon_html = if let Some(ref favicon) = config.favicon {
        format!(r#"<link rel="icon" href="{}/assets/{}">"#, base_path_url, favicon)
    } else {
        String::new()
    };

    let theme_toggle_html = r#"
        <div id="theme-toggle-container">
            <select id="theme-toggle" onchange="setTheme(this.value)">
                <option value="system">System</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
            </select>
        </div>
    "#;

    format!(
        r##"<!DOCTYPE html>
<html style="scroll-padding-top: 80px;">
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    {}
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css">
    <style>
        :root {{
            {}
            --sidebar-width: 260px;
            --minimap-width: 240px;
            --navbar-height: 60px;
        }}

        @media (prefers-color-scheme: dark) {{
            :root {{ {} }}
        }}

        [data-theme='light'] {{ {} }}
        [data-theme='dark'] {{ {} }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
            margin: 0;
            display: flex;
            background: var(--bg-color);
            color: var(--text-color);
            height: 100vh;
            overflow: hidden;
        }}

        #navbar {{
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            height: var(--navbar-height);
            background: var(--navbar-bg);
            border-bottom: 1px solid var(--border-color);
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 40px;
            z-index: 1000;
        }}

        #sidebar {{
            width: var(--sidebar-width);
            height: calc(100vh - var(--navbar-height));
            position: fixed;
            left: 0;
            top: var(--navbar-height);
            border-right: 1px solid var(--border-color);
            background: var(--sidebar-bg);
            overflow-y: auto;
            padding: 20px 0;
            box-sizing: border-box;
        }}

        #minimap {{
            width: var(--minimap-width);
            height: calc(100vh - var(--navbar-height));
            position: fixed;
            right: 0;
            top: var(--navbar-height);
            border-left: 1px solid var(--border-color);
            background: var(--sidebar-bg);
            overflow-y: auto;
            padding: 60px 20px 20px 20px;
            box-sizing: border-box;
        }}

        #main-content {{
            margin-left: var(--sidebar-width);
            {}
            padding: 2rem 4rem 10rem 4rem;
            flex-grow: 1;
            margin-top: var(--navbar-height);
            max-width: 1000px;
            position: relative;
            height: calc(100vh - var(--navbar-height));
            overflow-y: auto;
            box-sizing: border-box;
        }}

        #search-bar-trigger {{
            padding: 6px 12px;
            background: var(--search-bg);
            border: 1px solid var(--search-border);
            border-radius: 6px;
            color: var(--search-text);
            font-size: 13px;
            cursor: pointer;
            width: 200px;
            text-align: left;
            display: flex;
            justify-content: space-between;
            align-items: center;
            transition: all 0.2s;
        }}
        #search-bar-trigger:hover {{
            opacity: 0.8;
        }}

        #search-modal {{
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            background: rgba(0,0,0,0.5);
            z-index: 2000;
            justify-content: center;
            padding-top: 100px;
        }}
        #search-modal.visible {{ display: flex; }}

        #search-container {{
            background: var(--modal-bg);
            width: 600px;
            max-height: 400px;
            border-radius: 12px;
            box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
            display: flex;
            flex-direction: column;
            overflow: hidden;
            border: 1px solid var(--border-color);
        }}
        #search-input-wrapper {{
            padding: 15px;
            border-bottom: 1px solid var(--border-color);
        }}
        #search-input {{
            width: 100%;
            border: none;
            background: transparent;
            color: var(--text-color);
            font-size: 16px;
            outline: none;
            font-family: inherit;
        }}
        #search-results {{
            overflow-y: auto;
            padding: 10px 0;
        }}
        .search-result {{
            padding: 12px 20px;
            display: flex;
            justify-content: space-between;
            align-items: center;
            cursor: pointer;
            text-decoration: none;
            color: inherit;
        }}
        .search-result:hover, .search-result.selected {{
            background: var(--search-bg);
        }}
        .result-line {{
            font-size: 14px;
            color: var(--text-color);
            flex-grow: 1;
            margin-right: 20px;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }}
        .result-title {{
            font-size: 12px;
            color: var(--search-text);
            background: var(--search-bg);
            padding: 2px 8px;
            border-radius: 4px;
            white-space: nowrap;
        }}
        .result-line mark {{
            background: var(--primary-color);
            color: #fff;
            padding: 0 2px;
            border-radius: 2px;
        }}

        .sidebar-item, .toc-item {{
            display: block;
            padding: 8px 20px;
            text-decoration: none;
            color: var(--sidebar-item-color);
            font-size: 14px;
            transition: all 0.2s;
        }}
        .toc-item {{ padding: 4px 0; font-size: 13px; color: var(--toc-item-color); }}
        .sidebar-item:hover, .toc-item:hover {{ color: var(--primary-color); }}
        .sidebar-item.active {{
            background: var(--search-bg);
            color: var(--primary-color);
            font-weight: bold;
            border-right: 3px solid var(--primary-color);
        }}

        .sidebar-group {{ display: none; }}
        .sidebar-group.expanded {{ display: block; }}

        h1, h2, h3, h4, h5, h6 {{ color: var(--h-color); margin-top: 1.5em; scroll-margin-top: 40px; }}
        p {{ line-height: 1.6; }}
        li {{ margin-bottom: 8px; }}
        #main-content a {{
            color: inherit;
            text-decoration: underline;
            cursor: pointer;
        }}
        #main-content a:hover {{
            text-decoration: none;
        }}
        code {{
            background: var(--code-bg);
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
            border-bottom: 1px solid var(--border-color);
        }}
        #version-selector-container select {{
            width: 100%;
            padding: 8px 12px;
            border: 1px solid var(--search-border);
            border-radius: 6px;
            background: var(--search-bg);
            color: var(--text-color);
            outline: none;
            font-size: 13px;
            cursor: pointer;
            transition: all 0.2s;
            appearance: none;
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%236b7280'%3E%3Cpath stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M19 9l-7 7-7-7'%3E%3C/path%3E%3C/svg%3E");
            background-repeat: no-repeat;
            background-position: right 10px center;
            background-size: 14px;
        }}
        #version-selector-container select:hover {{
            opacity: 0.8;
        }}
        .toc-title {{
            font-size: 11px;
            text-transform: uppercase;
            letter-spacing: 1px;
            color: var(--search-text);
            margin-bottom: 10px;
            font-weight: bold;
        }}
        img {{
            max-width: 100%;
            height: auto;
            border-radius: 8px;
            margin: 1.5rem 0;
            box-shadow: 0 4px 12px rgba(0,0,0,0.05);
        }}

        #theme-toggle-container {{
            margin-left: 15px;
            display: flex;
            align-items: center;
        }}
        #theme-toggle-container select {{
            padding: 4px 8px;
            border: 1px solid var(--search-border);
            border-radius: 6px;
            background: var(--search-bg);
            color: var(--text-color);
            font-size: 12px;
            cursor: pointer;
            outline: none;
        }}
    </style>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
    <script>
        window.__verdocs_version = "{}";
        window.__verdocs_current_version_name = "{}";
        window.__verdocs_base_path = "{}";

        function setTheme(theme) {{
            if (theme === 'system') {{
                document.documentElement.removeAttribute('data-theme');
            }} else {{
                document.documentElement.setAttribute('data-theme', theme);
            }}
            localStorage.setItem('verdocs-theme', theme);
        }}
        const savedTheme = localStorage.getItem('verdocs-theme') || 'system';
        setTheme(savedTheme);

        setInterval(() => {{
            if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {{
                fetch(window.__verdocs_base_path + '/__verdocs/status')
                    .then(r => r.text())
                    .then(v => {{
                        if (v !== window.__verdocs_version) {{
                            location.reload();
                        }}
                    }});
            }}
        }}, 1000);

        function switchVersion(newVersion) {{
            const currentPath = window.location.pathname;
            const pathParts = currentPath.split('/');
            const versionIndex = pathParts.findIndex(part => part === window.__verdocs_current_version_name);
            if (versionIndex !== -1) {{
                pathParts[versionIndex] = newVersion;
                window.location.pathname = pathParts.join('/');
            }} else {{
                window.location.pathname = window.__verdocs_base_path + '/' + newVersion + '/home';
            }}
        }}

        let searchIndex = null;
        let selectedIndex = -1;

        async function initSearch() {{
            if (searchIndex) return;
            const response = await fetch(window.__verdocs_base_path + `/search-index/${{window.__verdocs_current_version_name.replace(/\./g, '-')}}.json`);
            searchIndex = await response.json();
        }}

        function openSearch() {{
            const modal = document.getElementById('search-modal');
            modal.classList.add('visible');
            document.getElementById('search-input').focus();
            initSearch();
        }}

        function closeSearch() {{
            const modal = document.getElementById('search-modal');
            modal.classList.remove('visible');
        }}

        function performSearch(query) {{
            const resultsContainer = document.getElementById('search-results');
            resultsContainer.innerHTML = '';
            selectedIndex = -1;

            if (query.length < 3) return;

            const results = [];
            searchIndex.forEach(page => {{
                const lines = page.content.split('\n');
                lines.forEach(line => {{
                    const index = line.toLowerCase().indexOf(query.toLowerCase());
                    if (index !== -1) {{
                        results.push({{
                            title: page.title,
                            route: page.route,
                            line: line.trim(),
                            highlightIndex: index,
                            queryLength: query.length
                        }});
                    }}
                }});
            }});

            const topResults = results.slice(0, 5);
            topResults.forEach((res, i) => {{
                const div = document.createElement('div');
                div.className = 'search-result';

                const before = res.line.substring(0, res.highlightIndex);
                const match = res.line.substring(res.highlightIndex, res.highlightIndex + res.queryLength);
                const after = res.line.substring(res.highlightIndex + res.queryLength);

                div.innerHTML = `
                    <div class="result-line">${{escapeHtml(before)}}<mark>${{escapeHtml(match)}}</mark>${{escapeHtml(after)}}</div>
                    <div class="result-title">${{escapeHtml(res.title)}}</div>
                `;
                div.onclick = () => window.location.href = window.__verdocs_base_path + `/${{window.__verdocs_current_version_name}}/${{res.route}}`;
                resultsContainer.appendChild(div);
            }});
        }}

        function escapeHtml(text) {{
            const div = document.createElement('div');
            div.innerText = text;
            return div.innerHTML;
        }}

        document.addEventListener('keydown', (e) => {{
            if ((e.metaKey || e.ctrlKey) && e.key === 'k') {{
                e.preventDefault();
                openSearch();
            }}
            if (e.key === 'Escape') closeSearch();

            const modal = document.getElementById('search-modal');
            if (modal.classList.contains('visible')) {{
                const results = document.querySelectorAll('.search-result');
                if (e.key === 'ArrowDown') {{
                    e.preventDefault();
                    selectedIndex = Math.min(selectedIndex + 1, results.length - 1);
                    updateSelection(results);
                }} else if (e.key === 'ArrowUp') {{
                    e.preventDefault();
                    selectedIndex = Math.max(selectedIndex - 1, 0);
                    updateSelection(results);
                }} else if (e.key === 'Enter' && selectedIndex !== -1) {{
                    results[selectedIndex].click();
                }}
            }}
        }});

        function updateSelection(results) {{
            results.forEach((r, i) => {{
                if (i === selectedIndex) r.classList.add('selected');
                else r.classList.remove('selected');
            }});
        }}

        document.addEventListener('DOMContentLoaded', () => {{
            document.getElementById('theme-toggle').value = savedTheme;

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
    <div id="search-modal" onclick="if(event.target === this) closeSearch()">
        <div id="search-container">
            <div id="search-input-wrapper">
                <input type="text" id="search-input" placeholder="Search documentation..." oninput="performSearch(this.value)">
            </div>
            <div id="search-results"></div>
        </div>
    </div>
    <div id="navbar">
        <div id="navbar-left">
            {}
        </div>
        <div id="navbar-right" style="display: flex; align-items: center;">
            <div id="search-bar-trigger" onclick="openSearch()">
                <span>Search...</span>
                <span style="font-size: 11px; color: #bbb;">⌘K</span>
            </div>
            {}
        </div>
    </div>
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
</html>"##,
        config.title,
        favicon_html,
        light_vars,
        dark_vars,
        light_vars,
        dark_vars,
        main_content_style,
        version_timestamp,
        current_version,
        base_path_url,
        navbar_logo_html,
        theme_toggle_html,
        version_options,
        sidebar_html,
        body,
        minimap_html
    )
}
fn render_sidebar(
    items: &[SidebarItem],
    version: &str,
    current_route: &str,
    depth: usize,
    base_path_url: &str,
) -> String {
    let mut html = String::new();
    let indent = depth * 15 + 20;
    for item in items {
        let is_active = item.route == current_route;
        let is_descendant = current_route.starts_with(&item.route) && !item.route.is_empty();
        let active_class = if is_active { "active" } else { "" };
        html.push_str(&format!(
            r#"<a href="{}/{}/{}" class="sidebar-item {}" style="padding-left: {}px;">{}</a>"#,
            base_path_url, version, item.route, active_class, indent, item.title
        ));
        if !item.children.is_empty() {
            let expanded_class = if is_descendant || is_active {
                "expanded"
            } else {
                ""
            };
            html.push_str(&format!(
                r#"<div class="sidebar-group {}">"#,
                expanded_class
            ));
            html.push_str(&render_sidebar(
                &item.children,
                version,
                current_route,
                depth + 1,
                base_path_url,
            ));
            html.push_str("</div>");
        }
    }
    html
}
