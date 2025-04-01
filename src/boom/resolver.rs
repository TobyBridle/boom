use concat_string::concat_string;

use crate::cache::{REDIRECT_LIST, get_bang};

use super::{
    DEFAULT_SEARCH_INDEXES, DEFAULT_SEARCH_TEMPLATE, parse_bangs::parse_bang_indexes,
    parse_templates::parse_template_indexes,
};

/// Resolves a url-decoded query to its correct search url (or None)
pub fn resolve(query: &str) -> String {
    assert!(!query.is_empty());

    if let Some(bang_idx) = parse_bang_indexes(query) {
        let bang = &query[bang_idx.to_indices(0)][1..];

        let query = concat_string!(
            query[..bang_idx.start],
            query[bang_idx.end.min(bang.len())..]
        );
        let encoded_query = urlencoding::encode(query.as_str());

        let redirect_idx = get_bang(bang)
            .unwrap()
            .expect("Expected all entries to be available in cache");
        let mut template = REDIRECT_LIST.read().unwrap()[redirect_idx]
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
            concat_string!(template.to_string(), encoded_query)
        }
    } else {
        concat_string!(
            DEFAULT_SEARCH_TEMPLATE[..DEFAULT_SEARCH_INDEXES.start],
            urlencoding::encode(query),
            DEFAULT_SEARCH_TEMPLATE[DEFAULT_SEARCH_INDEXES.end..]
        )
    }
}
