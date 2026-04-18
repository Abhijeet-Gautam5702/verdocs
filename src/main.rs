mod clean;
mod cli;
mod config;
mod generator;
mod init;
mod server;

use crate::clean::clean_project;
use crate::cli::{Cli, Commands};
use crate::generator::generate_site;
use crate::init::init_project;
use crate::server::start_server;
use anyhow::Result;
use clap::Parser;
use notify::{Event, RecursiveMode, Watcher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { path } => {
            init_project(path)?;
            let abs_path = std::fs::canonicalize(path)?;
            println!("Project initialized successfully at: {:?}", abs_path);
        }
        Commands::Generate { path } => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            generate_site(path, now)?;
            println!("Site generated successfully from: {:?}", path);
        }
        Commands::Preview { path, port } => {
            let version = Arc::new(AtomicU64::new(
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            ));

            // Initial generation
            generate_site(path, version.load(Ordering::SeqCst))?;

            // Canonicalize paths for reliable filtering
            let abs_root = std::fs::canonicalize(path)?;
            let abs_out = abs_root.join("out");

            let abs_root_clone = abs_root.clone();
            let abs_out_clone = abs_out.clone();
            let version_clone = version.clone();

            // Set up file watcher
            let mut watcher =
                notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
                    match res {
                        Ok(event) => {
                            let mut should_reload = false;
                            for p in &event.paths {
                                let abs_p = if p.is_absolute() {
                                    p.clone()
                                } else {
                                    abs_root_clone.join(p)
                                };

                                // 1. Ignore if inside 'out/'
                                if abs_p.starts_with(&abs_out_clone) {
                                    continue;
                                }

                                // 2. Get path relative to the root to avoid matching parent directory names
                                if let Ok(rel_p) = abs_p.strip_prefix(&abs_root_clone) {
                                    let first_component = rel_p.components().next();
                                    if let Some(std::path::Component::Normal(name)) = first_component {
                                        let s = name.to_str().unwrap_or("");
                                        // ONLY watch versioned folders or config.yml
                                        if s.starts_with('v') || s == "config.yml" {
                                            should_reload = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            if should_reload {
                                let new_v = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();
                                if let Err(e) = generate_site(&abs_root_clone, new_v) {
                                    println!("Error re-generating site: {}", e);
                                } else {
                                    version_clone.store(new_v, Ordering::SeqCst);
                                    println!("Site re-generated due to changes.");
                                }
                            }
                        }
                        Err(e) => println!("watch error: {:?}", e),
                    }
                })?;

            watcher.watch(&abs_root, RecursiveMode::Recursive)?;

            println!("Starting preview server at http://localhost:{} ...", port);
            start_server(&abs_out, *port, version)?;
        }
        Commands::Clean { path } => {
            clean_project(path)?;
            println!("Project cleaned successfully at: {:?}", path);
        }
    }

    Ok(())
}
