mod test {
    #[allow(unused_imports)]
    use crate::Assets;

    #[allow(unused_imports)]
    use crate::ConfigBuilder;

    #[allow(unused_imports)]
    use std::path::PathBuf;

    #[allow(unused_imports)]
    use crate::{BangCustomConfig, ServerConfig};

    #[test]
    fn test_config_parse() {
        let config = String::from_utf8(
            Assets::get("default_config.toml")
                .expect("default config should exist.")
                .data
                .into_owned(),
        )
        .unwrap();

        let parsed_config = toml::from_str::<ConfigBuilder>(&config)
            .expect("Config should be properly formatted.")
            .build();
        dbg!(&parsed_config);
        assert_eq!(parsed_config.server, ServerConfig::default());
        assert_eq!(
            parsed_config.bangs.default_search_template,
            "https://google.com/search?q={{{s}}}"
        );
        assert!(parsed_config.bangs.sources[0].required);
        assert_eq!(
            parsed_config.bangs.sources[0].filepath,
            PathBuf::from("~/.cache/boom/bangs.json")
        );
        assert_eq!(
            parsed_config.bangs.sources[0].remote,
            Some("https://duckduckgo.com/bang.js".to_string())
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
