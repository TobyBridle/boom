use std::{
    collections::HashMap,
    net::IpAddr,
    path::PathBuf,
    process::{self, exit},
};

use toml_edit::Item;

use super::{BangConfig, BangCustomConfig, BangDefaultConfig, Config, ServerConfig};

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

fn parse_server_config(table: &Item) -> ServerConfig {
    let default = ServerConfig::default();

    let address = table.get("address").map_or(default.address, |address| {
        address
            .as_str()
            .unwrap_or_else(|| {
                eprintln!("Expected [server.address] to be an IpAddr. Got {address:?}");
                exit(1)
            })
            .parse::<IpAddr>()
            .unwrap_or_else(|_| {
                eprintln!("Expected [server.address] to be an IpAddr. Got {address:?}");
                exit(1)
            })
    });

    let port = table.get("port").map_or(default.port, |port| {
        u16::try_from(port.as_integer().unwrap_or_else(|| {
            eprintln!(
                "Expected [server.port] to be an unsigned 16-bit integer (e.g 3000). Got {port:?}"
            );
            exit(1);
        }))
        .unwrap_or_else(|_| {
            eprintln!(
                "Expected [server.port] to be an unsigned 16-bit integer (e.g 3000). Got {port:?}"
            );
            exit(1);
        })
    });

    let wait_for_internet =
        table
            .get("wait_for_internet")
            .map_or(default.wait_for_internet, |wait_for_internet| {
                wait_for_internet.as_bool().unwrap_or_else(|| {
                eprintln!(
                    "Expected [server.wait_for_internet] to be a boolean. Got {wait_for_internet:?}"
                );
                exit(1);
            })
            });

    let is_secure = table
        .get("is_secure")
        .map_or(default.is_secure, |is_secure| {
            is_secure.as_bool().unwrap_or_else(|| {
                eprintln!("Expected [server.is_secure] to be a boolean. Got {is_secure:?}");
                exit(1);
            })
        });

    ServerConfig {
        address,
        port,
        wait_for_internet,
        is_secure,
    }
}

#[allow(clippy::too_many_lines)]
fn parse_bang_config(config: &Item) -> BangConfig {
    let default = BangConfig::default();

    // [bangs] root
    let default_search_template = config.get("default_search_template").map_or(
        default.default_search_template,
        |default_template| {
            default_template.as_str().unwrap_or_else(|| {
                eprintln!("[bangs.default_search_template] is expected to be a string. Got {default_template:?}");
                exit(1)
            }).to_string()
        },
    );

    // [bangs.default]
    let default_bangs = get_table!(config, "default", "[bangs.default] must be a table.")
        .map_or_else(BangDefaultConfig::default, |default_bangs| {
            let enabled =
                default_bangs
                    .get("enabled")
                    .map_or(default.default.enabled, |enabled| {
                        enabled.as_bool().unwrap_or_else(|| {
eprintln!("[bangs.default.enabled] is expected to be a boolean. Got {enabled:?}");
                        exit(1);
                    })
                    });
            let filepath =
                default_bangs
                    .get("filepath")
                    .map_or(default.default.filepath, |filepath| {
                        filepath.as_str().unwrap_or_else(|| {
eprintln!("[bangs.default.filepath] is expected to be a string. Got {filepath:?}");
                        exit(1);
                    }).strip_prefix("~/").map_or_else(
                            || PathBuf::from(filepath.as_str().unwrap()),
                            |stripped| {
                                let home = home::home_dir().expect("$HOME should be accessible");
                                home.join(stripped)
                            },
                        )
                    });
            let remote =
                default_bangs
                    .get("remote")
                    .map_or(default.default.remote, |remote| {
                        remote.as_str().unwrap_or_else(|| {
eprintln!("[bangs.default.remote] is expected to be a string. Got {remote:?}");
                        exit(1);
                    }).to_string()
                    });

            BangDefaultConfig {
                enabled,
                filepath,
                remote,
            }
        });

    // [bangs.custom]
    let custom_bangs = get_table!(config, "custom", "[bangs.custom] must be a table").map_or_else(
        HashMap::new,
        |custom_table| {
            let mut map = HashMap::new();

            for (key, val) in custom_table {
                if let Item::Table(inner) = val {
                    let template = inner
                        .get("template")
                        .unwrap_or_else(|| {
                            eprintln!("Template should exist for [bangs.custom.\"{key}\"]");
                            exit(1)
                        })
                        .as_str()
                        .unwrap();

                    let trigger = inner
                        .get("trigger")
                        .unwrap_or_else(|| {
                            eprintln!("Trigger should exist for [bangs.custom.\"{key}\"]");
                            exit(1)
                        })
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
                        for (k, v) in inline {
                            temp.insert(k, v.clone().into());
                        }

                        let template = temp
                            .get("template")
                            .unwrap_or_else(|| {
                                eprintln!("Template should exist for [bangs.custom.\"{key}\"]");
                                exit(1)
                            })
                            .as_str()
                            .unwrap();

                        let trigger = temp
                            .get("trigger")
                            .unwrap_or_else(|| {
                                eprintln!("Trigger should exist for [bangs.custom.\"{key}\"]");
                                exit(1)
                            })
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
        },
    );

    BangConfig {
        default_search_template,
        default: default_bangs,
        custom: custom_bangs,
    }
}

/// Parses a config in the form of a string
///
/// # Panics
/// If the contents of the string is not valid TOML/UTF-8
pub fn parse_config(config: &str) -> Config {
    let config = config
        .parse::<toml_edit::DocumentMut>()
        .unwrap_or_else(|e| {
            eprintln!("Config should be valid TOML. Error: {e:?}");
            process::exit(1);
        });

    let server_config = parse_section!(
        config,
        "server",
        parse_server_config,
        ServerConfig::default()
    );

    let bang_config = parse_section!(config, "bangs", parse_bang_config, BangConfig::default());

    Config {
        server: server_config,
        bangs: bang_config,
    }
}

mod test {
    #[allow(unused_imports)]
    use std::path::PathBuf;

    #[allow(unused_imports)]
    use crate::{BangCustomConfig, ServerConfig, parse_config::parse_config};

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

        let parsed_config = parse_config(config);
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
