use std::{
    collections::HashMap,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerConfig {
    pub address: IpAddr,
    pub port: u16,
    pub wait_for_internet: bool,
    pub is_secure: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 3000,
            wait_for_internet: false,
            is_secure: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BangConfig {
    pub default_search_template: String,
    pub default: BangDefaultConfig,
    pub custom: HashMap<String, BangCustomConfig>,
}

impl Default for BangConfig {
    fn default() -> Self {
        Self {
            default_search_template: "https://google.com/search?q={{{s}}}".to_string(),
            default: BangDefaultConfig::default(),
            custom: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BangDefaultConfig {
    pub enabled: bool,
    pub filepath: PathBuf,
    pub remote: String,
}

impl Default for BangDefaultConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            filepath: PathBuf::new(),
            remote: "https://duckduckgo.com/bang.js".to_string(),
        }
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
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Merge, Deserialize)]
pub struct BangConfigBuilder {
    #[merge(strategy = merge::option::overwrite_none)]
    pub default_search_template: Option<String>,
    #[merge(strategy = merge::option::overwrite_none)]
    pub default: Option<BangDefaultConfigBuilder>,
    #[merge(strategy = merge::hashmap::overwrite)]
    pub custom: HashMap<String, BangCustomConfig>,
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Merge, Deserialize)]
pub struct BangDefaultConfigBuilder {
    #[merge(strategy = merge::option::overwrite_none)]
    pub enabled: Option<bool>,
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

    pub fn disable_default_bangs(&mut self) -> &mut Self {
        self.bangs
            .get_or_insert_default()
            .default
            .get_or_insert_default()
            .enabled = Some(false);
        self
    }

    pub fn set_bang_cache<P: Into<PathBuf>>(&mut self, path: P) -> &mut Self {
        self.bangs
            .get_or_insert_default()
            .default
            .get_or_insert_default()
            .filepath = Some(path.into());
        self
    }

    pub fn set_cache_origin<P: Into<String>>(&mut self, origin: P) -> &mut Self {
        self.bangs
            .get_or_insert_default()
            .default
            .get_or_insert_default()
            .remote = Some(origin.into());
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
        }
    }
}

impl From<BangConfig> for BangConfigBuilder {
    fn from(config: BangConfig) -> Self {
        Self {
            default_search_template: Some(config.default_search_template),
            default: Some(config.default.into()),
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
            default: builder
                .default
                .unwrap_or_else(|| default.default.into())
                .into(),
            custom: builder.custom,
        }
    }
}

impl From<BangDefaultConfig> for BangDefaultConfigBuilder {
    fn from(config: BangDefaultConfig) -> Self {
        Self {
            enabled: Some(config.enabled),
            filepath: Some(config.filepath),
            remote: Some(config.remote),
        }
    }
}

impl From<BangDefaultConfigBuilder> for BangDefaultConfig {
    fn from(builder: BangDefaultConfigBuilder) -> Self {
        let default = Self::default();
        Self {
            enabled: builder.enabled.unwrap_or(default.enabled),
            filepath: builder.filepath.unwrap_or(default.filepath),
            remote: builder.remote.unwrap_or(default.remote),
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
