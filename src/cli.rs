use std::{env, net::IpAddr, path::PathBuf};

use boom_config::{ConfigBuilder, ConfigSource};
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

        /// Wait for the connection to be valid
        /// before attempting to start serving
        /// instead of panicking
        #[arg(short = 'w', long, default_value_t = false)]
        await_internet: bool,
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
    pub(crate) const fn setup_type(&self) -> SetupMode {
        match self {
            Self::Serve { .. } => SetupMode::All,
            Self::Resolve { no_cache, .. } => {
                if *no_cache {
                    SetupMode::All
                } else {
                    SetupMode::Caches
                }
            }
            Self::Validate { .. } => SetupMode::NoSetup,
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

impl ConfigSource for Args {
    fn read_into_builder(&self) -> Result<ConfigBuilder, Box<dyn std::error::Error>> {
        let mut builder = ConfigBuilder::new();
        if let LaunchType::Serve {
            addr,
            port,
            await_internet,
        } = self.launch
        {
            builder.set_port(port).set_address(addr);
            if await_internet {
                builder.wait_for_internet();
            }
        }
        Ok(builder)
    }
}
