use std::{env, path::PathBuf};

use clap::{Parser, Subcommand, command};
use serde::Serialize;

fn get_default_bang_path() -> PathBuf {
    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(&home_dir)
        .join(".config")
        .join("boom")
        .join("default_bangs.json")
}

#[derive(Subcommand, Clone, Default, Debug, Serialize)]
pub enum LaunchType {
    /// Launch the redirecting server
    #[default]
    Serve,

    /// Test the resolution of a search query
    Resolve {
        /// The full search query.
        /// E.g, !d blazing fast
        #[arg(required = true)]
        search_query: String,
    },
}

/// Processor for [DuckDuckGo Bang](https://duckduckgo.com/bangs) Parsing
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub launch: LaunchType,
    /// Path to a JSON file containing bang commands
    #[arg(short, long, default_value = get_default_bang_path().into_os_string())]
    pub bang_commands: PathBuf,
}
