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

#[derive(Subcommand, Clone, Debug, Serialize)]
pub enum LaunchType {
    /// Launch the redirecting server
    /// =============================
    /// To make use of the server, set the search engine on your
    /// browser to match the address & port here.
    /// For example, with the default config, you would add
    /// the search engine as `http://localhost:3000?q=%s`
    #[command(verbatim_doc_comment)]
    Serve {
        /// The address to run the server on
        #[arg(long, default_value = String::from("127.0.0.1"))]
        addr: String,
        /// The port to run the server on
        /// e.g 3000 -> localhost:3000
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },

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
