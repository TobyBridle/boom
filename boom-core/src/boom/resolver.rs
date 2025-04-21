use boom_config::Config;
use concat_string::concat_string;

use crate::{
    boom::Match,
    cache::{get_bang, get_redirects},
};

use super::{parse_bangs::parse_bang_indexes, parse_templates::parse_template_indexes};

/// Resolves a url-decoded query to its correct search url
///
/// # Panics
/// Panics if the query is an empty string.
#[must_use]
pub fn resolve(query: &str, config: &Config) -> String {
    assert!(!query.is_empty());

    let template = config.bangs.default_search_template.as_str();
    let indexes = parse_template_indexes(template).map_or_else(Match::default, |matches| {
        matches.first().copied().unwrap_or_default()
    });

    parse_bang_indexes(query).map_or_else(
        || {
            concat_string!(
                template[..indexes.start],
                urlencoding::encode(query).replace("%2F", "/"),
                template[indexes.end..]
            )
        },
        |bang_idx| {
            let bang = &query[bang_idx.start + 1..bang_idx.end];

            let query = concat_string!(
                query[..(bang_idx.start).max(1) - 1],
                &query[(bang_idx.end + 1).clamp(1, query.len())..]
            );
            let encoded_query = urlencoding::encode(query.as_str()).replace("%2F", "/");

            let redirect_idx = match get_bang(bang).unwrap() {
                Some(idx) => idx,
                None => {
                    eprintln!(
                        "Bang ({bang}) could not be found in cache. Assuming default search."
                    );
                    return concat_string!(
                        template[..indexes.start],
                        encoded_query,
                        template[indexes.end..]
                    );
                }
            };
            let mut template = get_redirects().expect("Redirect list should be initialised")
                [redirect_idx]
                .url_template
                .clone();

            if let Some(template_idx) = parse_template_indexes(template.as_str()) {
                let mut result = String::with_capacity(1024);
                for templates in template_idx {
                    if !templates.is_empty() {
                        result.push_str(&template[..templates.start]);
                        result.push_str(&encoded_query);
                        template = template[..templates.start].to_string();
                    }
                }

                result
            } else {
                template
            }
        },
    )
}

mod tests {
    #[allow(unused_imports)]
    use boom_config::Config;

    #[allow(unused_imports)]
    use crate::{
        Redirect,
        boom::{Match, resolver::resolve},
        cache::{init_list, insert_bang},
    };

    #[test]
    fn test_resolve_no_bang() {
        let query = "test query";
        assert_eq!(
            resolve(query, &Config::default()),
            "https://google.com/search?q=test%20query"
        );
    }

    #[test]
    fn test_resolve_bang_prefix() {
        init_list(
            vec![Redirect {
                short_name: "YouTube".to_string(),
                trigger: "yt".to_string(),
                url_template: "https://youtube.com/results?search_query={{{s}}}".to_string(),
            }],
            true,
        )
        .unwrap();
        insert_bang("yt".to_string(), 0).unwrap();

        let query = "!yt test query";
        assert_eq!(
            resolve(query, &Config::default()),
            "https://youtube.com/results?search_query=test%20query"
        );
    }

    #[test]
    fn test_resolve_bang_suffix() {
        init_list(
            vec![Redirect {
                short_name: "YouTube".to_string(),
                trigger: "yt".to_string(),
                url_template: "https://youtube.com/results?search_query={{{s}}}".to_string(),
            }],
            true,
        )
        .unwrap();
        let query = "test query !yt";
        assert_eq!(
            resolve(query, &Config::default()),
            "https://youtube.com/results?search_query=test%20query"
        );
    }

    #[test]
    fn test_resolve_bang_slash() {
        init_list(
            vec![Redirect {
                short_name: "GitHub".to_string(),
                trigger: "gh".to_string(),
                url_template: "https://github.com/{{{s}}}".to_string(),
            }],
            true,
        )
        .unwrap();
        insert_bang("gh".to_string(), 1).unwrap();

        let query = "tobybridle/boom !gh";
        assert_eq!(
            resolve(query, &Config::default()),
            "https://github.com/tobybridle/boom"
        );
    }
}
