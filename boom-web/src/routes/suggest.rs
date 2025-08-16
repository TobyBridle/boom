use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use reqwest::Client;
use serde_json::Value;
use tracing::error;

use crate::{AppState, routes::index::SearchParams};

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
            |json| (StatusCode::OK, headers.clone(), Json(json)),
        )
    } else {
        error!("Could not parse response json.");
        bad_request()
    }
}
