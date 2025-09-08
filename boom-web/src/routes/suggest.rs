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
#[cfg(feature = "history-suggestions")]
use tracing::debug;

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
                    use boom_core::cache::SEARCH_HISTORY_CACHE;

                    let mut j = serde_json::from_value::<Suggestions>(json)
                        .expect("API result should be valid suggestions");
                    SEARCH_HISTORY_CACHE
                        .try_read()
                        .unwrap()
                        .iter()
                        .for_each(|h| {
                            dbg!(&h.query.1, &query);
                            if h.query.1.starts_with(query.as_str()) {
                                debug!("Injecting suggestion into those from {url}");
                                debug!("Query: {query}");
                                debug!("Suggestion: {:?}", &h.query.1);
                                j.suggestions.insert(0, h.query.1.clone())
                            }
                        });
                    serde_json::to_value(j).unwrap()
                };
                (StatusCode::OK, headers.clone(), Json(json))
            },
        )
    } else {
        error!("Could not parse response json.");
        bad_request()
    }
}
