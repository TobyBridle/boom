use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use serde::Deserialize;
use toml_edit::Item;

use crate::cli::get_default_bang_path;

macro_rules! get_table {
    ($parent:expr, $key:expr, $err_msg:expr) => {{
        $parent.get($key).map(|item| {
            assert!(item.is_table(), $err_msg);
            item.as_table().unwrap()
        })
    }};
}

macro_rules! parse_section {
    ($config:expr, $key:expr, $parser:expr, $default:expr) => {{ $config.get($key).map($parser).unwrap_or_else(|| $default) }};
}

#[derive(Deserialize, Default, Debug, PartialEq)]
pub struct Config {
    server: ServerConfig,
    bangs: BangConfig,
}

#[derive(Deserialize, Debug, PartialEq)]
struct ServerConfig {
    address: IpAddr,
    port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 3000,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct BangConfig {
    default_search_template: String,
    default: BangDefaultConfig,
    custom: HashMap<String, BangCustomConfig>,
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

#[derive(Deserialize, Debug, PartialEq)]
struct BangDefaultConfig {
    enabled: bool,
    filepath: PathBuf,
    remote: String,
}

impl Default for BangDefaultConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            filepath: get_default_bang_path(),
            remote: "https://duckduckgo.com/bang.js".to_string(),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
struct BangCustomConfig {
    template: String,
    trigger: String,
}

fn parse_server_config(table: &Item) -> ServerConfig {
    let default = ServerConfig::default();

    let address = if let Some(address) = table.get("address") {
        assert!(
            address.is_str(),
            "[server.address] is expected to be an IpAddr"
        );
        let ip = address.as_str().unwrap().parse::<IpAddr>();
        assert!(
            ip.is_ok(),
            "[server.address] is expected to be an IpAddr. Got {ip:?}"
        );
        ip.unwrap()
    } else {
        default.address
    };

    let port = if let Some(port) = table.get("port") {
        assert!(
            port.is_integer(),
            "[server.port] is expected to be a u16. Got {port:?}"
        );
        port.as_integer().unwrap() as u16
    } else {
        default.port
    };

    ServerConfig { address, port }
}

fn parse_bang_config(config: &Item) -> BangConfig {
    let default = BangConfig::default();

    // [bangs] root
    let default_search_template = if let Some(default_template) =
        config.get("default_search_template")
    {
        assert!(
            default_template.is_str(),
            "[bangs.default_search_template] is expected to be a string. Got {default_template:?}"
        );
        default_template.as_str().unwrap().to_string()
    } else {
        default.default_search_template
    };

    // [bangs.default]
    let default_bangs = if let Some(default_bangs) =
        get_table!(config, "default", "[bangs.default] must be a table.")
    {
        let enabled = if let Some(enabled) = default_bangs.get("enabled") {
            assert!(
                enabled.is_bool(),
                "[bangs.default.enabled] is expected to be a boolean. Got {enabled:?}"
            );
            enabled.as_bool().unwrap()
        } else {
            default.default.enabled
        };
        let filepath = if let Some(filepath) = default_bangs.get("filepath") {
            assert!(
                filepath.is_str(),
                "[bangs.default.filepath] is expected to be a string. Got {filepath:?}"
            );
            PathBuf::from(filepath.as_str().unwrap())
        } else {
            default.default.filepath
        };
        let remote = if let Some(remote) = default_bangs.get("remote") {
            assert!(
                remote.is_str(),
                "[bangs.default.remote] is expected to be a string. Got {remote:?}"
            );
            remote.as_str().unwrap().to_string()
        } else {
            default.default.remote
        };

        BangDefaultConfig {
            enabled,
            filepath,
            remote,
        }
    } else {
        BangDefaultConfig::default()
    };

    // [bangs.custom]
    let custom_bangs = if let Some(custom_table) =
        get_table!(config, "custom", "[bangs.custom] must be a table")
    {
        let mut map = HashMap::new();

        for (key, val) in custom_table.iter() {
            if let Item::Table(inner) = val {
                let template = inner
                    .get("template")
                    .expect("Template should exist for any given bang")
                    .as_str()
                    .unwrap();

                let trigger = inner
                    .get("trigger")
                    .expect("Trigger should exist for any given bang")
                    .as_str()
                    .unwrap();

                if !template.is_empty() && !trigger.is_empty() {
                    map.insert(
                        key.to_string(),
                        BangCustomConfig {
                            template: template.to_string(),
                            trigger: trigger.to_string(),
                        },
                    );
                }
            } else if let Item::Value(_) = val {
                if let Some(inline) = val.as_value().and_then(|v| v.as_inline_table()) {
                    let mut temp = toml_edit::Table::new();
                    for (k, v) in inline.iter() {
                        temp.insert(k, v.clone().into());
                    }

                    let template = temp
                        .get("template")
                        .expect("Template should exist for any given bang")
                        .as_str()
                        .unwrap();

                    let trigger = temp
                        .get("trigger")
                        .expect("Trigger should exist for any given bang")
                        .as_str()
                        .unwrap();

                    if !template.is_empty() && !trigger.is_empty() {
                        map.insert(
                            key.to_string(),
                            BangCustomConfig {
                                template: template.to_string(),
                                trigger: trigger.to_string(),
                            },
                        );
                    }
                }
            }
        }

        map
    } else {
        HashMap::new()
    };

    BangConfig {
        default_search_template,
        default: default_bangs,
        custom: custom_bangs,
    }
}

pub fn parse_config(config: String) -> Result<Config, Box<dyn std::error::Error>> {
    let config = config
        .parse::<toml_edit::DocumentMut>()
        .expect("Config should be valid TOML");

    let server_config = parse_section!(
        config,
        "server",
        parse_server_config,
        ServerConfig::default()
    );

    let bang_config = parse_section!(config, "bangs", parse_bang_config, BangConfig::default());

    Ok(Config {
        server: server_config,
        bangs: bang_config,
    })
}

mod test {
    #[allow(unused_imports)]
    use crate::config::{BangCustomConfig, BangDefaultConfig, ServerConfig, parse_config};
    #[allow(unused_imports)]
    use std::{net::IpAddr, path::PathBuf};

    #[test]
    fn test_config_parse() {
        let config = r#"
            [server]
            address = "127.0.0.1"
            port = "abc"

            [bangs]
            # The entirety of `{{{s}}}` will be replaced with the search term
            default_search_template = "https://google.com/search?q={{{s}}}"

            # Set the path to a default bang file
            [bangs.default]
            # Whether to bother requesting the bangs or not
            enabled = true
            filepath = "~/.cache/boom/bangs.json"
            # Where to fetch the bangs from
            remote = "https://duckduckgo.com/bang.js"

            [bangs.custom]
            boomdev = { template = "https://github.com/tobybridle/boom", trigger = "boomdev" }
            # ^ shortname

            # You can also set them like this
            [bangs.custom.amazingdev]
            # `!amazedev boom` resolves to the url for this project!
            template = "https://github.com/tobybridle/{{{s}}}"
            trigger = "amazedev"
        "#;

        let parsed_config = parse_config(config.to_string()).unwrap();
        dbg!(&parsed_config);
        assert_eq!(parsed_config.server, ServerConfig::default());
        assert_eq!(
            parsed_config.bangs.default_search_template,
            "https://google.com/search?q={{{s}}}"
        );
        assert!(parsed_config.bangs.default.enabled);
        assert_eq!(
            parsed_config.bangs.default.filepath,
            PathBuf::from("~/.cache/boom/bangs.json")
        );
        assert_eq!(
            parsed_config.bangs.default.remote,
            "https://duckduckgo.com/bang.js"
        );
        assert_eq!(
            parsed_config.bangs.custom.get("boomdev"),
            Some(&BangCustomConfig {
                trigger: "boomdev".to_string(),
                template: "https://github.com/tobybridle/boom".to_string(),
            })
        );
        assert_eq!(
            parsed_config.bangs.custom.get("amazingdev"),
            Some(&BangCustomConfig {
                trigger: "amazedev".to_string(),
                template: "https://github.com/tobybridle/{{{s}}}".to_string(),
            })
        )
    }
}
