mod test {
    #[allow(unused_imports)]
    use crate::ConfigBuilder;

    #[allow(unused_imports)]
    use std::path::PathBuf;

    #[allow(unused_imports)]
    use crate::{BangCustomConfig, ServerConfig};

    #[test]
    fn test_config_parse() {
        let config = r#"
            [server]
            address = "127.0.0.1"
            port = 3000

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

        let parsed_config = toml::from_str::<ConfigBuilder>(config)
            .expect("Config should be properly formatted.")
            .build();
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
