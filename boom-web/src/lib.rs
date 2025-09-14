//! # About
//! `boom-web` leverages [`axum`] to allow for a high-speed and resource-efficient web server.
//! Whilst [`boom_core`] provides the tools to crunch data, `boom-web` provides the user-facing
//! functions to display awesome web pages.

#[cfg(feature = "history")]
use std::time::Duration;
use std::{
    net::{IpAddr, SocketAddr},
    sync::{Arc, RwLock},
};

use axum::{Router, routing::get};
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_template::engine::Engine;
use boom_config::{Config, ConfigBuilder, get_default_config_path};
use boom_core::boom::update_bangs_from_config::update_bangs_from_config;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use notify::{RecommendedWatcher, Watcher};
use routes::{bangs::list_bangs, index::redirector, opensearch::opensearch};
use rust_embed::RustEmbed;
use tokio::net::TcpListener;
use tower::util::Either;
use tracing::{error, info};

#[cfg(feature = "history")]
use crate::routes::history::list_history;
use crate::routes::suggest::suggest;

#[cfg(feature = "history")]
mod history;

mod routes;

type AppEngine = Engine<Handlebars<'static>>;

pub struct EitherResponse<A, B>(pub Either<A, B>);

impl<A, B> IntoResponse for EitherResponse<A, B>
where
    A: IntoResponse,
    B: IntoResponse,
{
    fn into_response(self) -> Response {
        match self.0 {
            Either::Left(a) => a.into_response(),
            Either::Right(b) => b.into_response(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    engine: AppEngine,
    shared_config: Arc<RwLock<Config>>,
}

#[derive(RustEmbed)]
#[folder = "assets/"]
/// Assets bundled directly into the binary.
struct Assets;

async fn asset_handler(Path(path): Path<String>) -> impl IntoResponse {
    match Assets::get(&path) {
        Some(asset) => {
            let mime = mime_guess::from_path(&path).first_or_text_plain();
            Response::builder()
                .status(200)
                .header("Content-Type", mime.as_ref())
                .body(axum::body::Body::from(asset.data))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("404 not found".into())
            .unwrap(),
    }
}

/// Serve the web server on `address` and `port`
///
/// # Panics
/// Panics if the server could not bind to the desired address/port.
pub async fn serve(address: IpAddr, port: u16, config: &Config) {
    info!(name:"Boom", "Starting Web Server on {}:{}", address, port);

    let mut hbs = Handlebars::new();
    hbs.register_helper("json", Box::new(json_helper));

    hbs.register_template_string("/", include_str!("../assets/index.html"))
        .expect("Template should be syntactically correct");

    hbs.register_template_string("/bangs", include_str!("../assets/bangs/index.html"))
        .expect("Template should be syntactically correct");

    #[cfg(feature = "history")]
    hbs.register_template_string("/history", include_str!("../assets/history/index.html"))
        .expect("Template should be syntactically correct");

    let shared_config = RwLock::from(config.clone());

    let state = AppState {
        engine: Engine::from(hbs),
        shared_config: Arc::new(shared_config),
    };

    // NOTE: Hot-reloading only works using the default config path!
    let shared_config = Arc::clone(&state.shared_config);

    #[cfg(feature = "history")]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("Process should be able to listen to signals");
        let mut sigint =
            signal(SignalKind::interrupt()).expect("Process should be able to listen to signals");
        let mut sigusr1 = signal(SignalKind::user_defined1())
            .expect("Process should be able to listen to signals");

        tokio::spawn(async move {
            use tokio::time::{Instant, interval_at};

            use crate::history::save_history;
            let mut history_save_interval = interval_at(
                Instant::now() + Duration::from_secs(60),
                Duration::from_secs(60),
            );
            loop {
                history_save_interval.tick().await;
                save_history().await;
            }
        });

        tokio::spawn(async move {
            loop {
                use std::process::exit;

                use tokio::select;

                use crate::history::save_history;

                select! {
                    _ = sigint.recv() => {
                        info!("Attempting to save history before quitting");
                        save_history().await;
                        exit(1);
                    }
                    _ = sigterm.recv() => {
                        info!("Attempting to save history before quitting");
                        save_history().await;
                        exit(1);
                    }
                    _ = sigusr1.recv() => {
                        info!("Force saving history");
                        save_history().await;
                    }
                }
            }
        });
    }

    tokio::spawn(async move {
        let config_path = shared_config.read().map_or_else(
            |_| get_default_config_path(),
            |cfg| cfg.config_source.clone(),
        );

        info!("Awaiting changes on {}", config_path.display());

        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher =
            RecommendedWatcher::new(tx, notify::Config::default().with_compare_contents(true))
                .unwrap();
        watcher
            .watch(&config_path, notify::RecursiveMode::NonRecursive)
            .unwrap();
        for res in rx {
            match res {
                Ok(event) => {
                    if !matches!(
                        event.kind,
                        notify::EventKind::Modify(notify::event::ModifyKind::Data(_)),
                    ) {
                        continue;
                    }

                    if let Ok(mut write_lock) = shared_config.write() {
                        let config = ConfigBuilder::new()
                            .add_source(&config_path)
                            .to_owned()
                            .build();
                        *write_lock = config;
                    }

                    let config_bangs = Arc::new(shared_config.read().unwrap().bangs.clone());
                    update_bangs_from_config(
                        config_bangs,
                        Arc::new(RwLock::new(vec![])),
                        true,
                        true,
                    )
                    .await;
                }
                Err(e) => error!("Watch Error: {e:?}"),
            }
        }
    });

    let mut router = Router::new()
        .route("/", get(redirector))
        .route("/bangs", get(list_bangs))
        .route("/suggest", get(suggest))
        .route("/opensearch.xml", get(opensearch))
        .route("/assets/{*path}", get(asset_handler)) // serve embedded files
        .route(
            "/sw.js",
            get(|| async { asset_handler(Path("bangs/sw.js".to_string())).await }),
        );

    #[cfg(feature = "history")]
    {
        router = router.route("/history", get(list_history));
    }

    let addr = SocketAddr::new(address, port);
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            return error!(name:"Boom", "Failed to bind to address {addr}. Reason: {e}");
        }
    };
    info!(name:"Boom", "Server running on {addr}");
    axum::serve(listener, router.with_state(state).into_make_service())
        .await
        .unwrap();
}

/// Allows JSON to be passed nicely into Handlebars templates.
///
/// # Errors
/// If the output cannot be written to
pub fn json_helper(
    h: &Helper<'_>,
    _: &Handlebars<'_>,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> HelperResult {
    // Take the first parameter to the helper
    if let Some(param) = h.param(0) {
        // Serialize it into JSON
        let json = serde_json::to_string(param.value()).unwrap_or_else(|_| "null".to_string());
        out.write(&json)?;
    } else {
        out.write("null")?;
    }

    Ok(())
}
