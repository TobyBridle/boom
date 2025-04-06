use std::path::PathBuf;

use clap::{Parser, Subcommand, command};
use serde::Serialize;

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
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub launch: LaunchType,
    /// Path to a JSON file containing bang commands
    #[arg(short, long, default_value = None)]
    pub bang_commands: Option<PathBuf>,
}
