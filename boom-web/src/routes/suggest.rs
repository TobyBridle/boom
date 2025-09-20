use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use reqwest::Client;

#[cfg(feature = "history-suggestions")]
use serde::{Deserialize, Serialize};
use serde_json::Value;

use tracing::error;

use crate::{AppState, routes::index::SearchParams};

#[cfg(feature = "history-suggestions")]
#[derive(Debug, Serialize, Deserialize)]
struct Suggestions {
    keyword: String,
    suggestions: Vec<String>,
}

/// [`suggest`] provides search suggestions for the browser, acting as a proxy for an existing
/// suggestions provider.
///
/// > **NOTE**: Some browsers, especially those which seek to enhance privacy for users, such as
/// > LibreWolf, may disable search suggestions by default.
/// > On Firefox-based browsers, it should be possible to enable the feature via the `#about:preferences#search` settings page.
/// > For those using Chromium-based browsers, the equivalent would be `chrome://settings/syncSetup`
pub async fn suggest(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    // Common error response
    let bad_request = || {
        (
            StatusCode::BAD_REQUEST,
            headers.clone(),
            Json(serde_json::json!([])),
        )
    };

    // Ensure we have a query string
    let Some(query) = params.query else {
        return bad_request();
    };

    // Build URL from config
    let url = {
        let cfg = state
            .shared_config
            .read()
            .expect("Shared Config should not be poisoned");
        cfg.server
            .search_suggestions
            .replace("{searchTerms}", &query)
    };

    // Perform HTTP request
    if let Ok(res) = Client::new().get(&url).send().await {
        (res.json::<Value>().await).map_or_else(
            |_| {
                error!("Could not parse response json.");
                bad_request()
            },
            |json| {
                #[cfg(feature = "history-suggestions")]
                let json = {
                    use boom_core::SourceIdentifier;
                    use boom_core::cache::SEARCH_HISTORY_CACHE;

                    let mut j = serde_json::from_value::<Suggestions>(json)
                        .expect("API result should be valid suggestions");
                    {
                        let mut cache = SEARCH_HISTORY_CACHE.try_write().unwrap();
                        cache.sort_by(|a, b| {
                            // Check if a or b matches the source identifier
                            let param_si = match params.source_identifier {
                                Some(ref si) => si,
                                None => &SourceIdentifier::Empty,
                            };

                            let a_matches = &a.source_identifier == param_si;
                            let b_matches = &b.source_identifier == param_si;

                            use std::cmp::Ordering;

                            match (a_matches, b_matches) {
                                (true, false) => Ordering::Greater, // a goes before b
                                (false, true) => Ordering::Less,    // b goes before a
                                _ => a.source_identifier.cmp(&b.source_identifier),
                            }
                        });

                        cache.iter().for_each(|h| {
                            if h.query.1.starts_with(query.as_str()) {
                                j.suggestions.insert(0, h.query.1.clone());
                            }
                        });
                    }
                    serde_json::json!([j.keyword, j.suggestions])
                };
                (StatusCode::OK, headers.clone(), Json(json))
            },
        )
    } else {
        error!("Could not parse response json.");
        bad_request()
    }
}
