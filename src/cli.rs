use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "verdocs")]
#[command(version)]
#[command(about = "A static documentation site generator with versioning support", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Host {
    Vps,
    Vercel,
    GhPages,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new documentation project
    Init {
        /// The directory path where the documentation project should be initialized
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Generate the documentation site
    Generate {
        /// The source directory path for the documentation project
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The hosting platform for optimization
        #[arg(short, long, value_enum, default_value = "vps")]
        host: Host,
    },
    /// Preview the documentation site in your browser
    Preview {
        /// The source directory path for the documentation project
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Port to run the server on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    /// Clean up the project by removing generated files and config
    Clean {
        /// The directory path to clean
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Remove versioned directories as well
        #[arg(short, long)]
        full: bool,
    },
    /// Update Verdocs to the latest version
    SelfUpdate,
    /// Uninstall Verdocs from your system
    Uninstall,
}
