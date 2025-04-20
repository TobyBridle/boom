use std::{env, net::IpAddr, path::PathBuf};

use boom_config::Config;
use clap::{Parser, Subcommand, command};
use serde::Serialize;

#[must_use]
pub fn get_default_config_path() -> PathBuf {
    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(&home_dir)
        .join(".config")
        .join("boom")
        .join("config.toml")
}

#[must_use]
pub fn get_default_bang_path() -> PathBuf {
    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(&home_dir)
        .join(".cache")
        .join("boom")
        .join("bangs.json")
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
        addr: IpAddr,
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

        /// Redownloads any required bangs instead of relying on the cache
        #[arg(long, default_value_t = false)]
        no_cache: bool,
    },

    /// Validate the configuration
    Validate {
        #[arg(short, default_value_t = false)]
        verbose: bool,
    },
}

#[derive(PartialEq, Eq)]
pub(crate) enum SetupMode {
    All,
    Caches,
    NoSetup,
}

impl LaunchType {
    pub(crate) fn setup_type(&self) -> SetupMode {
        match self {
            LaunchType::Serve { .. } => SetupMode::All,
            LaunchType::Resolve { no_cache, .. } => {
                if *no_cache {
                    SetupMode::All
                } else {
                    SetupMode::Caches
                }
            }
            LaunchType::Validate { .. } => SetupMode::NoSetup,
        }
    }
}

/// Processor for [DuckDuckGo Bang](https://duckduckgo.com/bangs) Parsing
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub launch: LaunchType,

    /// Path to the configuration file to validate
    #[arg(short, long, default_value = get_default_config_path().into_os_string())]
    pub config: PathBuf,
}

#[must_use]
pub const fn merge_with_config(args: &Args, mut config: Config) -> Config {
    match args.launch {
        LaunchType::Serve {
            addr,
            port,
            await_internet,
        } => {
            config.server.address = addr;
            config.server.port = port;
            config
        }
        _ => config,
    }
}
