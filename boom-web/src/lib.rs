use std::{
    net::{IpAddr, SocketAddr},
    sync::{Arc, RwLock},
};

use axum::response::{IntoResponse, Response};
use axum::{Router, routing::get};
use axum_template::engine::Engine;
use boom_config::Config;
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use routes::{bangs::list_bangs, index::redirector, opensearch::opensearch};
use tokio::net::TcpListener;
use tower::util::Either;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{error, info};

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

/// Serve the web server on `address` and `port`
///
/// # Panics
/// Panics if the server could not bind to the desired address/port.
pub async fn serve(address: IpAddr, port: u16, config: &Config) {
    info!(name:"Boom", "Starting Web Server on {}:{}", address, port);

    let mut hbs = Handlebars::new();
    hbs.register_helper("json", Box::new(json_helper));

    hbs.register_template_string("/bangs", include_str!("../assets/bangs/index.html"))
        .expect("Template should be syntactically correct");

    let router = Router::new()
        .route("/", get(redirector))
        .route("/bangs", get(list_bangs))
        .route("/opensearch.xml", get(opensearch))
        .nest_service("/sw.js", ServeFile::new("boom-web/assets/bangs/sw.js"))
        .nest_service("/assets", ServeDir::new("boom-web/assets"))
        .with_state(AppState {
            engine: Engine::from(hbs),
            shared_config: Arc::new(RwLock::new(config.clone())),
        });

    let addr = SocketAddr::new(address, port);
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            return error!(name:"Boom", "Failed to bind to address {addr}. Reason: {e}");
        }
    };
    info!(name:"Boom", "Server running on {addr}");
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}

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
