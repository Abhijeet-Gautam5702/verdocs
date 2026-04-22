mod clean;
mod cli;
mod config;
mod generator;
mod init;
mod server;

use crate::clean::clean_project;
use crate::cli::{Cli, Commands, Host};
use crate::config::Config;
use crate::generator::generate_site;
use crate::init::init_project;
use crate::server::start_server;
use anyhow::Result;
use clap::Parser;
use notify::{Event, RecursiveMode, Watcher};
use std::fs;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const INSTALL_COMMAND: &str = "curl -fsSL https://raw.githubusercontent.com/Abhijeet-Gautam5702/verdocs/main/scripts/install.sh | bash";

fn main() -> Result<()> {
    let cli = Cli::parse();
    let version = env!("CARGO_PKG_VERSION");

    match &cli.command {
        Commands::Init { path } => {
            init_project(path)?;
            let abs_path = std::fs::canonicalize(path)?;
            println!("Project initialized successfully at: {:?}", abs_path);
        }
        Commands::Generate { path, host } => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            generate_site(path, now, host.clone())?;
            println!("Site generated successfully from: {:?}", path);
        }
        Commands::Preview { path, port } => {
            let config = Config::load(path)?;
            let version_ts = Arc::new(AtomicU64::new(
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            ));

            // Initial generation
            generate_site(path, version_ts.load(Ordering::SeqCst), Host::Vps)?;

            // Canonicalize paths for reliable filtering
            let abs_root = std::fs::canonicalize(path)?;
            let abs_out = abs_root.join("out");

            let abs_root_clone = abs_root.clone();
            let abs_out_clone = abs_out.clone();
            let version_clone = version_ts.clone();

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
                                    if let Some(std::path::Component::Normal(name)) =
                                        first_component
                                    {
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
                                if let Err(e) = generate_site(&abs_root_clone, new_v, Host::Vps) {
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

            let base_path = config.base_path.clone().unwrap_or_default();
            let base_path = if base_path.is_empty() {
                "".to_string()
            } else {
                format!("/{}", base_path.trim_matches('/'))
            };

            let url = format!("http://localhost:{}{}/index.html", port, base_path);
            println!("Starting preview server at {} ...", url);
            start_server(path, *port, version_ts, config)?;
        }
        Commands::Clean { path, full } => {
            clean_project(path, *full)?;
            let abs_path = std::fs::canonicalize(path)?;
            println!("Project cleaned successfully at: {:?}", abs_path);
        }
        Commands::Uninstall => {
            println!("Uninstalling verdocs v{}...", version);
            let home_dir = dirs::home_dir().unwrap_or_default();
            let verdocs_dir_path = home_dir.join(".verdocs");

            // remove config/analytics dir
            if verdocs_dir_path.exists() {
                println!("Removing directory: {}", verdocs_dir_path.display());
                fs::remove_dir_all(&verdocs_dir_path)?;
            }

            // remove binary
            let binary_path = std::env::current_exe()?;
            println!("Removing binary: {}", binary_path.display());
            match fs::remove_file(&binary_path) {
                Ok(()) => {
                    println!("verdocs v{} uninstalled successfully", version);
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        eprintln!("[ERROR] Uninstall Failed: PERMISSION DENIED");
                        println!("Try running with sudo:");
                        println!("  sudo verdocs uninstall");
                    } else {
                        eprintln!("[ERROR] Uninstall Failed: {}", e);
                    }
                    return Err(e.into());
                }
            }
        }
        Commands::SelfUpdate => {
            println!("Updating verdocs...");
            match Command::new("sh").arg("-c").arg(INSTALL_COMMAND).status() {
                Ok(status) => {
                    if status.success() {
                        println!("verdocs updated successfully");
                    } else {
                        eprintln!("[ERROR] Update script failed with status: {}", status);
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        eprintln!("[ERROR] Failed to update verdocs: PERMISSION DENIED");
                        println!("Try running with sudo:");
                        println!("  sudo verdocs self-update");
                    } else {
                        eprintln!("[ERROR] Failed to update verdocs: {}", e);
                    }
                    return Err(e.into());
                }
            }
        }
    }

    Ok(())
}
