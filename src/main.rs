use std::{io, time::Instant};

use boom::parse_bangs;
use cache::init_list;
use ntex::web::{self};
use routes::index::redirector;
use tracing::{Level, error, info};
pub mod boom;
pub mod cache;
pub mod routes;

const ADDR: &str = "127.0.0.1";
const PORT: u16 = 3000;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(if cfg!(debug_assertions) {
            Level::DEBUG
        } else {
            Level::INFO
        })
        .with_ansi(true)
        .compact()
        .with_writer(io::stderr)
        .init();

    info!(name: "Boom", "Parsing Bangs!");
    let now = Instant::now();
    let bangs = parse_bangs(None)
        .map_err(|e| {
            error!("Could not parse bangs! {:?}", e);
        })
        .unwrap();
    info!(
        name: "Boom",
        "Parsed {} bangs in {:?}!",
        bangs.len(),
        Instant::now().duration_since(now)
    );

    init_list(bangs, false).ok();

    info!(name:"Boom", "Starting Web Server on {}:{}", ADDR, PORT);
        .bind((ADDR, PORT))?
        .run()
        .await
}
