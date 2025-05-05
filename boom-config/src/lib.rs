use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use serde::Deserialize;
pub mod parse_config;
pub mod read_config;

#[derive(Clone, Deserialize, Default, Debug, PartialEq, Eq)]
pub struct Config {
    pub server: ServerConfig,
    pub bangs: BangConfig,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
pub struct ServerConfig {
    pub address: IpAddr,
    pub port: u16,
    pub wait_for_internet: bool,
    pub is_secure: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 3000,
            wait_for_internet: false,
            is_secure: false,
        }
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
pub struct BangCustomConfig {
    pub template: String,
    pub trigger: String,
}
