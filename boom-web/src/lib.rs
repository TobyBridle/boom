use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use axum::{Router, routing::get};
use axum_template::engine::Engine;
use boom_config::Config;
use handlebars::Handlebars;
use routes::{bangs::list_bangs, index::redirector};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::{error, info};

mod routes;

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Clone)]
pub struct AppState {
    engine: AppEngine,
    shared_config: Arc<RwLock<Config>>,
}

/// Serve the web server on `address` and `port`
///
/// # Panics
/// Panics if the server could not bind to the desired address/port.
pub async fn serve(address: &str, port: u16, config: Config) {
    info!(name:"Boom", "Starting Web Server on {}:{}", address, port);

    let mut hbs = Handlebars::new();
    hbs.register_template_string("/bangs", include_str!("../assets/bangs/index.html"))
        .expect("Template should be syntactically correct");

    let router = Router::new()
        .route("/", get(redirector))
        .route("/bangs", get(list_bangs))
        .nest_service("/assets", ServeDir::new("boom-web/assets"))
        .with_state(AppState {
            engine: Engine::from(hbs),
            shared_config: Arc::new(RwLock::new(config)),
        });

    let addr = SocketAddr::new(
        address.parse().expect("address should be a valid IpAddr"),
        port,
    );
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
