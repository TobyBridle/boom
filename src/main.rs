use std::time::Instant;

use boom::parse_bangs;
use cache::init_list;
use ntex::web::{self};
use routes::index::redirector;
use tracing::{Level, debug, error};
pub mod boom;
pub mod cache;
pub mod routes;

const ADDR: &str = "127.0.0.1";
const PORT: u16 = 3000;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_ansi(true)
        .compact()
        .init();

    debug!("Parsing Bangs!");
    let now = Instant::now();
    let bangs = parse_bangs(None)
        .map_err(|e| {
            error!("Could not parse bangs! {:?}", e);
        })
        .unwrap();
    debug!(
        "Parsed {} bangs in {:?}!",
        bangs.len(),
        Instant::now().duration_since(now)
    );

    init_list(bangs, false).ok();

    debug!("Starting Web Server on {}:{}", ADDR, PORT);
    web::HttpServer::new(|| web::App::new().service(redirector))
        .bind((ADDR, PORT))?
        .run()
        .await
}
