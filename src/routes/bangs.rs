use std::{
    sync::{LazyLock, RwLock},
    time::{Duration, Instant},
};

use crate::cache::REDIRECT_LIST;
use ntex::web;
use tracing::info;

static LAST_HTML_UPDATE: LazyLock<RwLock<Option<Instant>>> = LazyLock::new(|| RwLock::new(None));
static BANGS_HTML_CACHE: LazyLock<RwLock<String>> =
    LazyLock::new(|| RwLock::new("<h1>Bang cache not reloaded.</h1>".to_string()));
static HTML_STYLES: &str = "<style>table { font-family: monospace; } table th { text-align: left; padding: 1rem 0; font-size: 1.25rem; } table tr:nth-child(2n) { background: #161616; } table tr:nth-child(2n+1) { background: #181818; }</style>";

#[web::get("/bangs")]
pub async fn list_bangs() -> web::HttpResponse {
    let last_update = LAST_HTML_UPDATE
        .try_read()
        .ok()
        .and_then(|opt| opt.clone())
        .unwrap_or_else(|| {
            Instant::now()
                .checked_sub(Duration::from_secs(301))
                .unwrap()
        });

    let mut buffer = String::with_capacity(1024);

    if Instant::now().duration_since(last_update).as_secs() > 300 {
        info!(name: "Boom", "Updating /bangs");
        if let Ok(lock) = REDIRECT_LIST.try_read() {
            lock.iter().for_each(|redirection| {
                buffer.push_str(
                    format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                        redirection.short_name, redirection.trigger, redirection.url_template
                    )
                    .as_str(),
                )
            });

            return web::HttpResponse::Ok()
                .content_type("text/html")
                .body(format!(
                    "{}<table><tr><th>Abbr.</th><th>Short Code</th><th>URL Template</th></tr>{}{}",
                    HTML_STYLES, buffer, "</table>"
                ));
        }
    }

    web::HttpResponse::Ok()
        .content_type("text/html")
        .body(format!(
            "{}<table>{}{}",
            HTML_STYLES,
            BANGS_HTML_CACHE.try_read().map_or_else(
                |_| "<h1>Oops. Something went wrong on the server.</h1>".to_string(),
                |cached| { cached.clone() }
            ),
            "</table>"
        ))
}
