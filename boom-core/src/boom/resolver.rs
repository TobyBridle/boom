use concat_string::concat_string;

use crate::cache::{get_bang, get_redirects};

use super::{
    DEFAULT_SEARCH_INDEXES, DEFAULT_SEARCH_TEMPLATE, parse_bangs::parse_bang_indexes,
    parse_templates::parse_template_indexes,
};

/// Resolves a url-decoded query to its correct search url
///
/// # Panics
/// Panics if the query is an empty string.
#[must_use]
pub fn resolve(query: &str) -> String {
    assert!(!query.is_empty());

    parse_bang_indexes(query).map_or_else(
        || {
            concat_string!(
                DEFAULT_SEARCH_TEMPLATE[..DEFAULT_SEARCH_INDEXES.start],
                urlencoding::encode(query),
                DEFAULT_SEARCH_TEMPLATE[DEFAULT_SEARCH_INDEXES.end..]
            )
        },
        |bang_idx| {
            let bang = &query[bang_idx.start + 1..bang_idx.end];

            let query = concat_string!(
                query[..(bang_idx.start).max(1) - 1],
                &query[(bang_idx.end + 1).clamp(1, query.len())..]
            );
            let encoded_query = urlencoding::encode(query.as_str());

            let redirect_idx = get_bang(bang)
                .unwrap()
                .expect("Expected all entries to be available in cache");
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
    use crate::{
        Redirect,
        boom::resolver::resolve,
        cache::{init_list, insert_bang},
    };

    #[test]
    fn test_resolve_no_bang() {
        let query = "test query";
        assert_eq!(resolve(query), "https://google.com/search?q=test%20query");
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
            resolve(query),
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
            resolve(query),
            "https://youtube.com/results?search_query=test%20query"
        );
    }
}
