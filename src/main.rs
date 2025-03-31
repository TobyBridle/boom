use std::{io, time::Instant};

use boom::{DEFAULT_SEARCH_TEMPLATE, Match, parse_bangs::parse_bang_file, resolver::resolve};
use cache::init_list;
use clap::Parser;
use cli::LaunchType;
use ntex::web;
use routes::{bangs::list_bangs, index::redirector};
use tracing::{Level, error, info};
pub mod boom;
pub mod cache;
pub mod cli;
pub mod routes;

const ADDR: &str = "127.0.0.1";
const PORT: u16 = 3000;

#[macro_use(concat_string)]
extern crate concat_string;

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

    let args = cli::Args::parse();

    info!(name: "Boom", "Parsing Bangs!");
    let now = Instant::now();
    let bangs = parse_bang_file(None)
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

    match args.launch {
        LaunchType::Serve => {
            serve().await.unwrap();
        }
        LaunchType::Resolve { search_query } => {
            resolve(search_query.as_str()).await;
        }
    }

    Ok(())
}

pub async fn serve() -> Result<(), std::io::Error> {
    info!(name:"Boom", "Starting Web Server on {}:{}", ADDR, PORT);

    web::HttpServer::new(|| web::App::new().service(redirector).service(list_bangs))
        .bind((ADDR, PORT))?
        .run()
        .await
}
