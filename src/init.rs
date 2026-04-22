use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use crate::config::Config;

pub fn init_project(path: &PathBuf) -> Result<()> {
    // Check if project is already initialized
    if path.join("config.yml").exists() {
        return Err(anyhow::anyhow!(
            "Project already initialized at {:?}. Use 'verdocs clean' first if you want to reset.",
            path
        ));
    }

    fs::create_dir_all(path.join("assets"))?;
    fs::create_dir_all(path.join("search-index"))?;

    // Embed and write the logo
    let logo = include_bytes!("../resources/verdocs-logo.png");
    fs::write(path.join("assets/verdocs-logo.png"), logo)?;

    // Check if any versioned folders already exist
    let mut has_versioned_folders = false;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.starts_with('v') {
                            has_versioned_folders = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    if has_versioned_folders {
        println!("Existing versioned folders detected. Skipping sample folder creation.");
    } else {
        // Create v1.0.0 - Initial Release
        create_v1_0_0(path)?;
        
        // Create v1.1.0 - Feature Update
        create_v1_1_0(path)?;
    }

    let mut config = Config::default();
    config.navbar_logo = Some("verdocs-logo.png".to_string());
    config.favicon = Some("verdocs-logo.png".to_string());
    let config_yaml = serde_yaml::to_string(&config)?;
    fs::write(path.join("config.yml"), config_yaml)?;

    Ok(())
}

fn create_v1_0_0(root: &PathBuf) -> Result<()> {
    let v = root.join("v1.0.0");
    
    // Home
    write_doc(&v, "home/home.md", r##"# Welcome to Verdocs

Verdocs is a high-performance, version-aware documentation generator built in Rust. It's designed to help you create beautiful, searchable, and versioned documentation for your projects with minimal effort.

![Verdocs Logo](../../assets/verdocs-logo.png)

### Core Philosophy

Our goal is to provide a documentation tool that is:
*   **⚡ Blazing Fast**: Leverages Rust's performance for instant builds and site navigation.
*   **📦 Version-Native**: Documentation isn't just a snapshot; it's a history. We treat versioning as a first-class citizen.
*   **🔍 Searchable**: Integrated search that "just works" without complex server setups.

{TIP type="admonition" title="Quick Start"}
Ready to jump in? Go to the [Quick Start Guide](../getting-started/getting-started.md) to initialize your first project!
{/TIP}

### Key Features

1.  **Markdown Powered**: Use the familiar Markdown syntax you already know.
2.  **Live Reload**: See your changes in real-time as you edit. (Available in v1.1.0+)
3.  **Client-side Search**: Fast, responsive search across all versions.
4.  **Flexible Sidebar**: Automatically generated navigation based on your folder structure.
"##)?;

    // Getting Started
    write_doc(&v, "getting-started/getting-started.md", r##"# Getting Started

Follow these steps to get Verdocs up and running and create your first documentation site.

## Navigation
*   [Installation](./installation.md)
*   [Quick Start](./quick-start.md)

{NOTE type="admonition" title="Project Structure"}
Verdocs relies on a specific folder structure to manage versions and assets. This guide will walk you through the standard setup.
{/NOTE}
"##)?;

    write_doc(&v, "getting-started/installation.md", r##"# Installation

Getting Verdocs up and running is simple and straightforward.

## Prerequisites

Before installing Verdocs, ensure you have the following installed on your system:
*   **Rust Toolchain**: You'll need `cargo` and `rustc`. Get them at [rustup.rs](https://rustup.rs/).
*   **Git**: For cloning the repository.

## Installation Steps

Currently, Verdocs is installed by building from source. This ensures you have the latest performance optimizations for your architecture.

```bash
# Clone the repository
git clone https://github.com/verdocs/verdocs

# Navigate to the directory
cd verdocs

# Install the binary
cargo install --path .
```

{NOTE type="admonition" title="Path Configuration"}
Ensure that your Cargo bin directory (usually `~/.cargo/bin`) is in your system's `PATH` so you can run `verdocs` from anywhere.
{/NOTE}
"##)?;

    write_doc(&v, "getting-started/quick-start.md", r##"# Quick Start

Follow these steps to create your first documentation site with Verdocs in under a minute.

## 1. Initialize your project

Create a new directory for your documentation and run the `init` command:

```bash
mkdir my-docs
cd my-docs
verdocs init
```

This will create a basic structure with a sample `v1.0.0` and `v1.1.0`.

## 2. Start the development server

Run the following command to serve your documentation locally:

```bash
verdocs serve
```

By default, the server will be available at `http://localhost:3000`.

## 3. Build for production

When you're ready to deploy, run:

```bash
verdocs generate
```

This creates a `dist/` directory containing the static HTML, CSS, and JS files.

{IMPORTANT type="admonition" title="Production Ready"}
The output of the `generate` command is completely static and can be hosted on GitHub Pages, Vercel, Netlify, or any other static hosting provider.
{/IMPORTANT}
"##)?;

    // Features
    write_doc(&v, "features/features.md", r##"# Features

Verdocs comes packed with features designed to make documentation easy and beautiful.

## Core Features
*   [Admonitions](./admonitions.md)
*   [Search](./search.md)
*   [Live Reload (v1.1.0)](../v1.1.0/features/live-reload.md)
"##)?;

    write_doc(&v, "features/admonitions.md", r##"# Admonitions

Admonitions are a powerful way to draw attention to specific information using color-coded blocks.

## Usage

Verdocs uses a special syntax for admonitions that is compatible with most Markdown editors while allowing for rich styling.

```markdown
{TIP type="admonition" title="Pro Tip"}
This is how you write a tip!
{/TIP}
```

## Gallery

### Tip
{TIP type="admonition" title="Useful Tip"}
Use tips for helpful suggestions, shortcuts, or "did you know" facts.
{/TIP}

### Note
{NOTE type="admonition" title="Information"}
Notes are great for general information, background context, or side-bars.
{/NOTE}

### Warning
{WARN type="admonition" title="Warning"}
Use warnings to prevent common mistakes or draw attention to potential pitfalls.
{/WARN}

### Important
{IMPORTANT type="admonition" title="Critical"}
Use this for information that is absolutely essential for the user to understand.
{/IMPORTANT}

### Danger
{DANGER type="admonition" title="Danger Zone"}
Warn users about dangerous actions that could lead to data loss or system failure.
{/DANGER}

### Error
{ERROR type="admonition" title="Error"}
Indicate something that is invalid, has failed, or requires immediate correction.
{/ERROR}

## Inline Tags

You can also use these tags inline without the `type="admonition"` attribute for simple colored text: {TIP}Like this!{/TIP}
"##)?;

    write_doc(&v, "features/search.md", r##"# Search Functionality

Verdocs includes a powerful, zero-config search engine that works entirely on the client side.

## How it Works

When you run `verdocs generate` or while `verdocs serve` is running, Verdocs:
1.  Scans all your Markdown files.
2.  Indexes the headings and content.
3.  Generates a optimized JSON search index for each version.

## Usage

You can trigger the search interface in two ways:
1.  **Clicking**: Click the "Search..." bar at the top of the content area.
2.  **Keyboard**: Press `Cmd + K` (macOS) or `Ctrl + K` (Linux/Windows).

### Search Results

The search results show:
*   The matching line of text with the query highlighted.
*   The title of the page where the match was found.
*   Navigation to the result via mouse or keyboard (arrow keys + enter).

{NOTE type="admonition" title="Performance"}
Since the index is loaded once per version, searching is instantaneous and doesn't require any network requests after the initial load.
{/NOTE}
"##)?;

    // Core Concepts
    write_doc(&v, "core-concepts/core-concepts.md", r##"# Core Concepts

Understand the underlying principles and structure of Verdocs.

## Topics
*   [Versioning](./versioning.md)
"##)?;

    write_doc(&v, "core-concepts/versioning.md", r##"# Versioning

Versioning is the heart of Verdocs. We believe that documentation should evolve alongside your software.

## Folder-Based Versioning

In Verdocs, versions are defined by the top-level folders in your project directory (excluding `assets`, `search-index`, and `dist`).

```text
my-docs/
├── config.yml
├── v1.0.0/
│   └── home/home.md
└── v1.1.0/
    └── home/home.md
```

## Version Selector

Verdocs automatically detects these folders and generates a version selector in the sidebar. When a user switches versions, Verdocs attempts to keep them on the same relative page (e.g., switching from `v1.0.0/api/auth` to `v1.1.0/api/auth`).

## Versioning Strategy

We recommend using [Semantic Versioning](https://semver.org/) for your documentation folders, but Verdocs supports any naming convention you prefer (e.g., `latest`, `stable`, `v2-beta`).
"##)?;

    Ok(())
}

fn create_v1_1_0(root: &PathBuf) -> Result<()> {
    let v = root.join("v1.1.0");
    
    // Create base structure from v1.0.0 logic
    create_v1_0_0(root)?; 
    // We will now overwrite/add v1.1.0 specific files

    // Updated Home
    write_doc(&v, "home/home.md", r##"# Welcome to Verdocs v1.1.0

Verdocs continues to evolve! This version introduces exciting new features like Live Reload and enhanced custom theming.

![Verdocs Logo](../../assets/verdocs-logo.png)

{TIP type="admonition" title="What's New in v1.1.0?"}
We've added a **Live Reload** feature! No more manual refreshing. Check it out in the [Features section](../features/live-reload.md).
{/TIP}

### New in this version:
*   **Live Reload**: Automatic browser refresh on save.
*   **Custom Admonitions**: Define your own tags in `config.yml`.
*   **Advanced Layouts**: Better support for complex documentation structures.
"##)?;

    // Updated Features Index
    write_doc(&v, "features/features.md", r##"# Features

Verdocs v1.1.0 adds powerful new development tools.

## Core Features
*   [Admonitions](./admonitions.md)
*   [Search](./search.md)
*   [Live Reload](./live-reload.md) (New!)
"##)?;

    // New Feature: Live Reload
    write_doc(&v, "features/live-reload.md", r##"# Live Reload

Introduced in v1.1.0, Live Reload makes the documentation writing process seamless.

## How it works

The `verdocs serve` command now includes a small WebSocket-less script in the generated HTML.
1.  The server watches your file system for changes.
2.  When a file is saved, the server updates its internal state.
3.  The browser polls a small status endpoint every second.
4.  If a change is detected, the page reloads automatically.

## Usage

Simply run:
```bash
verdocs serve
```
And start editing your Markdown files. Your browser will reflect the changes almost instantly.

{NOTE type="admonition" title="No Configuration Needed"}
Live reload is enabled by default in development mode and is automatically stripped out during production builds (`verdocs generate`).
{/NOTE}
"##)?;

    // New Category: Advanced
    write_doc(&v, "advanced/advanced.md", r##"# Advanced Topics

Take your documentation to the next level with advanced configuration and theming.

## Topics
*   [Custom Theming](./custom-theming.md)
"##)?;

    write_doc(&v, "advanced/custom-theming.md", r##"# Custom Theming

Make your documentation reflect your brand with Verdocs' flexible theming system.

## The `config.yml` file

All theme configurations are handled in the `config.yml` at the root of your project.

```yaml
title: "My Brand Docs"
description: "Documentation for my amazing project"
theme:
  primary_color: "#722ed1"
  background_color: "#ffffff"
  text_color: "#1f1f1f"
  colors:
    tip: "#52c41a"
    note: "#1890ff"
    custom: "#eb2f96" # You can add custom admonition tags!
```

## Custom Admonitions

You aren't limited to the default tags. You can define any tag name and color in the `theme.colors` section, and use it like this:

```markdown
{CUSTOM type="admonition" title="Special Note"}
This uses the pink color defined in our config!
{/CUSTOM}
```

{NOTE type="admonition" title="Accessibility"}
When choosing colors, ensure there is sufficient contrast between the background and text colors to maintain accessibility.
{/NOTE}
"##)?;

    // Ensure v1.1.0 consistent structure for other folders
    write_doc(&v, "getting-started/getting-started.md", r##"# Getting Started

Installation remains unchanged in v1.1.0. Actually, we've improved the build speed by 20% in this version!

## Navigation
*   [Installation](./installation.md)
*   [Quick Start](./quick-start.md)
"##)?;

    write_doc(&v, "getting-started/installation.md", r##"# Installation

```bash
cargo install --path . --force
```
"##)?;

    // Copy remaining from v1.0.0 logic if they don't exist
    if !v.join("getting-started/quick-start.md").exists() {
        write_doc(&v, "getting-started/quick-start.md", "# Quick Start\nRefer to v1.0.0.")?;
    }
    if !v.join("features/admonitions.md").exists() {
        write_doc(&v, "features/admonitions.md", "# Admonitions\nRefer to v1.0.0.")?;
    }
    if !v.join("features/search.md").exists() {
        write_doc(&v, "features/search.md", "# Search\nRefer to v1.0.0.")?;
    }
    if !v.join("core-concepts/core-concepts.md").exists() {
        write_doc(&v, "core-concepts/core-concepts.md", "# Core Concepts\nRefer to v1.0.0.")?;
    }
    if !v.join("core-concepts/versioning.md").exists() {
        write_doc(&v, "core-concepts/versioning.md", "# Versioning\nRefer to v1.0.0.")?;
    }

    Ok(())
}

fn write_doc(version_path: &PathBuf, relative_path: &str, content: &str) -> Result<()> {
    let full_path = version_path.join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(full_path, content)?;
    Ok(())
}
