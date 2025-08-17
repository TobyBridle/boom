use std::{
    collections::HashMap,
    env,
    fmt::Display,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use merge::Merge;
use rust_embed::RustEmbed;
use serde::Deserialize;
use tracing::error;
pub mod parse_config;
pub mod read_config;

#[derive(RustEmbed)]
#[folder = "src/"]
pub struct Assets;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Config {
    pub server: ServerConfig,
    pub bangs: BangConfig,
}

/// Uses [`env::var`] to find the best place to store/find the config.
///
/// On UNIX systems, `boom` will attempt to use `$XDG_CONFIG_HOME/boom/config.toml`, with `$HOME/.config/boom/config.toml` as a fallback.
///
/// On Windows, `%USERPROFILE%` will be used.
///
/// In the event that the platform-specific approaches fall through, `boom` will use the current
/// directory with `./boom/config.toml`.
#[must_use]
pub fn get_default_config_path() -> PathBuf {
    let config_dir = if cfg!(unix) {
        env::var("XDG_CONFIG_HOME").map_or_else(
            |_| {
                env::var("HOME").map_or_else(
                    |_| PathBuf::from(".".to_string()),
                    |home| PathBuf::from(home).join(".config"),
                )
            },
            PathBuf::from,
        )
    } else {
        PathBuf::from(
            env::var("USERPROFILE").map_or_else(|_| ".".to_string(), |home| home + "\\.config"),
        )
    };
    config_dir.join("boom").join("config.toml")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerConfig {
    pub address: IpAddr,
    pub port: u16,
    pub wait_for_internet: bool,
    pub is_secure: bool,
    pub search_suggestions: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 3000,
            wait_for_internet: false,
            is_secure: false,
            search_suggestions: "https://search.brave.com/api/suggest?q={searchTerms}".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BangConfig {
    pub default_search_template: String,
    pub sources: Vec<BangSourceConfig>,
    pub custom: HashMap<String, BangCustomConfig>,
}

impl Default for BangConfig {
    fn default() -> Self {
        Self {
            default_search_template: "https://google.com/search?q={{{s}}}".to_string(),
            sources: vec![BangSourceConfig::default()],
            custom: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BangSourceConfig {
    pub required: bool,
    pub filepath: PathBuf,
    pub remote: Option<String>,
}

impl Default for BangSourceConfig {
    fn default() -> Self {
        Self {
            required: true,
            filepath: get_default_config_path(),
            remote: Some("https://duckduckgo.com/bang.js".to_string()),
        }
    }
}

impl Display for BangSourceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "\"{}\" {} [{}]",
            self.filepath.display(),
            self.remote.as_ref().unwrap_or(&String::new()),
            if self.required {
                "required"
            } else {
                "optional"
            }
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct BangCustomConfig {
    pub template: String,
    pub trigger: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Merge, Deserialize)]
pub struct ConfigBuilder {
    #[merge(strategy = merge::option::overwrite_none)]
    server: Option<ServerConfigBuilder>,
    #[merge(strategy = merge::option::overwrite_none)]
    bangs: Option<BangConfigBuilder>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Merge, Deserialize)]
pub struct ServerConfigBuilder {
    #[merge(strategy = merge::option::overwrite_none)]
    pub address: Option<IpAddr>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub port: Option<u16>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub wait_for_internet: Option<bool>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub is_secure: Option<bool>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub search_suggestions: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Merge, Deserialize)]
pub struct BangConfigBuilder {
    #[merge(strategy = merge::option::overwrite_none)]
    pub default_search_template: Option<String>,
    #[merge(strategy = merge::vec::append)]
    #[serde(rename = "source")]
    #[merge(strategy = merge::option::overwrite_none)]
    pub sources: Option<Vec<BangSourceConfigBuilder>>,
    #[merge(strategy = merge::hashmap::overwrite)]
    pub custom: HashMap<String, BangCustomConfig>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Merge, Deserialize)]
pub struct BangSourceConfigBuilder {
    #[merge(strategy = merge::option::overwrite_none)]
    pub required: Option<bool>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub filepath: Option<PathBuf>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub remote: Option<String>,
}

pub trait ConfigSource {
    /// Read a given source into a builder.
    /// # Errors
    /// Depending on the implementation. Suitable errors may be:
    /// - a non-existent file
    /// - a config which is not of the correct syntax
    fn read_into_builder(&self) -> Result<ConfigBuilder, Box<dyn std::error::Error>>;
}

impl ConfigBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_source<S: ConfigSource>(&mut self, source: &S) -> &mut Self {
        match source.read_into_builder() {
            Ok(builder) => {
                self.merge(builder);
            }
            Err(e) => error!(e),
        }
        self
    }

    pub fn set_address<A: Into<IpAddr>>(&mut self, addr: A) -> &mut Self {
        self.server.get_or_insert_default().address = Some(addr.into());
        self
    }

    pub fn set_port<P: Into<u16>>(&mut self, port: P) -> &mut Self {
        self.server.get_or_insert_default().port = Some(port.into());
        self
    }

    pub fn wait_for_internet(&mut self) -> &mut Self {
        self.server.get_or_insert_default().wait_for_internet = Some(true);
        self
    }

    pub fn secure(&mut self) -> &mut Self {
        self.server.get_or_insert_default().is_secure = Some(true);
        self
    }

    pub fn set_default_template<S: Into<String>>(&mut self, template: S) -> &mut Self {
        self.bangs.get_or_insert_default().default_search_template = Some(template.into());
        self
    }

    pub fn add_custom_bang<B: Into<String>, C: Into<BangCustomConfig>>(
        &mut self,
        bang_name: B,
        bang_config: C,
    ) -> &mut Self {
        self.bangs
            .get_or_insert_default()
            .custom
            .insert(bang_name.into(), bang_config.into());
        self
    }

    #[must_use]
    pub fn build(self) -> Config {
        Config {
            server: self.server.unwrap_or_default().into(),
            bangs: self.bangs.unwrap_or_default().into(),
        }
    }
}

impl From<ServerConfig> for ServerConfigBuilder {
    fn from(config: ServerConfig) -> Self {
        Self {
            address: Some(config.address),
            port: Some(config.port),
            wait_for_internet: Some(config.wait_for_internet),
            is_secure: Some(config.is_secure),
            search_suggestions: Some(config.search_suggestions),
        }
    }
}

impl From<ServerConfigBuilder> for ServerConfig {
    fn from(builder: ServerConfigBuilder) -> Self {
        let default = Self::default();
        Self {
            wait_for_internet: builder
                .wait_for_internet
                .unwrap_or(default.wait_for_internet),
            address: builder.address.unwrap_or(default.address),
            port: builder.port.unwrap_or(default.port),
            is_secure: builder.is_secure.unwrap_or(default.is_secure),
            search_suggestions: builder
                .search_suggestions
                .unwrap_or(default.search_suggestions),
        }
    }
}

impl From<BangConfig> for BangConfigBuilder {
    fn from(config: BangConfig) -> Self {
        Self {
            default_search_template: Some(config.default_search_template),
            sources: Some(config.sources.into_iter().map(Into::into).collect()),
            custom: config.custom,
        }
    }
}

impl From<BangConfigBuilder> for BangConfig {
    fn from(builder: BangConfigBuilder) -> Self {
        let default = Self::default();
        Self {
            default_search_template: builder
                .default_search_template
                .unwrap_or(default.default_search_template),
            sources: builder.sources.map_or_else(Vec::new, |sources| {
                sources.into_iter().map(Into::into).collect()
            }),
            custom: builder.custom,
        }
    }
}

impl From<BangSourceConfig> for BangSourceConfigBuilder {
    fn from(config: BangSourceConfig) -> Self {
        Self {
            required: Some(config.required),
            filepath: Some(config.filepath),
            remote: config.remote,
        }
    }
}

impl From<BangSourceConfigBuilder> for BangSourceConfig {
    fn from(builder: BangSourceConfigBuilder) -> Self {
        let default = Self::default();
        Self {
            required: builder.required.unwrap_or(default.required),
            filepath: builder.filepath.unwrap_or(default.filepath),
            remote: builder.remote,
        }
    }
}

impl From<Config> for ConfigBuilder {
    fn from(config: Config) -> Self {
        Self {
            server: Some(config.server.into()),
            bangs: Some(config.bangs.into()),
        }
    }
}
