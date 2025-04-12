use std::{
    fs::File,
    io::{self, BufReader, Read},
    time::Instant,
};

use boom::{parse_bangs::parse_bang_file, resolver::resolve};
use cache::{init_list, insert_bang};
use clap::Parser;
use cli::{LaunchType, get_default_bang_path};
use config::parse_config;
use ntex::web;
use routes::{bangs::list_bangs, index::redirector};
use tracing::{Level, error, info};
pub mod boom;
pub mod cache;
pub mod cli;
pub mod config;
pub mod routes;

extern crate concat_string;

fn setup() {
    info!(name: "Boom", "Parsing Bangs!");
    let now = Instant::now();
    let bangs = parse_bang_file(&get_default_bang_path())
        .map_err(|e| {
            error!("Could not parse bangs! {:?}", e);
        })
        .unwrap();
    let bangs_len = bangs.len();
    info!(
        name: "Boom",
        "Parsed {} bangs in {:?}!",
        bangs_len,
        Instant::now().duration_since(now)
    );

    init_list(bangs.clone(), false).ok();

    bangs.iter().enumerate().for_each(|(idx, bang)| {
        insert_bang(bang.trigger.clone(), idx).unwrap();
    });
}

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

    match args.launch {
        LaunchType::Serve { addr, port } => {
            setup();
            serve(addr.as_str(), port).await
        }
        LaunchType::Resolve { search_query } => {
            setup();
            println!("Resolved: {:?}", resolve(search_query.as_str()));
        }
        LaunchType::Validate { config, verbose } => {
            info!("Reading {}", config.display());
            let mut config_buffer = String::new();
            let mut breader = BufReader::new(File::open(config)?);
            breader.read_to_string(&mut config_buffer)?;
            match parse_config(config_buffer) {
                Ok(cfg) => {
                    if verbose {
                        dbg!(cfg);
                    }
                    info!("Parsed config with no errors.")
                }
                Err(e) => error!(e),
            }
        }
    }

    Ok(())
}

/// Serve the web server on [ADDR]:[PORT]
///
/// # Panics
/// Panics if the server could not bind to the desired address/port.
pub async fn serve(address: &str, port: u16) {
    info!(name:"Boom", "Starting Web Server on {}:{}", address, port);

    web::HttpServer::new(move || web::App::new().service(redirector).service(list_bangs))
        .bind((address, port))
        .expect("Address and port should be valid with no other applications using the same port.")
        .run()
        .await
        .unwrap_or_else(|_| panic!("Could not bind to {address}:{port}"));
}
