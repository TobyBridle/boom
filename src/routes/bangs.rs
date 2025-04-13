use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
    time::{Duration, Instant},
};

use crate::{AppEngine, AppState, boom::Redirect, cache::get_redirects};
use axum::{extract::State, response::IntoResponse};
use axum_template::RenderHtml;
use handlebars::Handlebars;
use serde::Serialize;
use tracing::info;

static LAST_HTML_UPDATE: LazyLock<RwLock<Option<Instant>>> = LazyLock::new(|| RwLock::new(None));
static BANGS_HTML_CACHE: LazyLock<RwLock<String>> =
    LazyLock::new(|| RwLock::new("<h1>Bang cache not reloaded.</h1>".to_string()));
static HTML_STYLES: &str = "<style></style>";

#[derive(Serialize, Debug)]
struct TemplateData {
    bangs: Vec<Redirect>,
}

pub async fn list_bangs(State(state): State<AppState>) -> impl IntoResponse {
    let last_update = LAST_HTML_UPDATE
        .try_read()
        .ok()
        .and_then(|opt| *opt)
        .unwrap_or_else(|| {
            Instant::now()
                .checked_sub(Duration::from_secs(301))
                .unwrap()
        });

    let data = TemplateData {
        bangs: get_redirects().unwrap().clone(),
    };

    dbg!(&data);
    RenderHtml("/bangs", state.engine, data)
    // if last_update.elapsed().as_secs() > 300 {
    //     info!(name: "Boom", "Updating /bangs");
    //     if let Ok(lock) = get_redirects() {
    //         lock.iter().for_each(|redirection| {
    //             buffer.push_str(
    //                 format!(
    //                     "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
    //                     redirection.short_name, redirection.trigger, redirection.url_template
    //                 )
    //                 .as_str(),
    //             );
    //         });
    //
    //         return Html(format!(
    //             "{}<table><tr><th>Abbr.</th><th>Short Code</th><th>URL Template</th></tr>{}{}",
    //             HTML_STYLES, buffer, "</table>"
    //         ));
    //     }
    // }
    //
    // Html(format!(
    //     "{}<table>{}{}",
    //     HTML_STYLES,
    //     BANGS_HTML_CACHE.try_read().map_or_else(
    //         |_| "<h1>Oops. Something went wrong on the server.</h1>".to_string(),
    //         |cached| { cached.clone() }
    //     ),
    //     "</table>"
    // ))
}
