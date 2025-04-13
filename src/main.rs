use std::{
    fs::File,
    io::{self, BufReader, Read},
    net::SocketAddr,
    time::Instant,
};

use axum::{Router, routing::get};
use boom::{
    Redirect, grab_remote_bangs::grab_remote, parse_bangs::parse_bang_file, resolver::resolve,
};
use cache::{init_list, insert_bang};
use clap::Parser;
use cli::LaunchType;
use config::{Config, parse_config::parse_config, read_config::read_config};
use routes::{bangs::list_bangs, index::redirector};
use tokio::net::TcpListener;
use tracing::{Level, error, info};
pub mod boom;
pub mod cache;
pub mod cli;
pub mod config;
pub mod routes;

extern crate concat_string;

#[inline]
async fn setup(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut bangs = if config.bangs.default.enabled {
        grab_remote(&config.bangs.default.remote, &config.bangs.default.filepath).await?;

        parse_bang_file(&config.bangs.default.filepath)
            .map_err(|e| {
                error!("Could not parse bangs! {:?}", e);
            })
            .unwrap()
    } else {
        info!("[bangs.default.enabled] = false");
        vec![]
    };

    info!(name: "Boom", "Parsing Bangs!");
    let now = Instant::now();

    dbg!(&config);
    config
        .bangs
        .custom
        .iter()
        .for_each(|(short_name, custom_config)| {
            bangs.push(Redirect {
                short_name: short_name.clone(),
                trigger: custom_config.trigger.clone(),
                url_template: custom_config.template.clone(),
            });
        });

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

    Ok(())
}

#[tokio::main]
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
    let config = read_config(&args.config).expect("Config path should be valid & readable.");

    match args.launch {
        LaunchType::Serve { addr, port } => {
            setup(config)
                .await
                .expect("Setup should be able to download bangs.");
            serve(addr.as_str(), port).await
        }
        LaunchType::Resolve { search_query } => {
            setup(config)
                .await
                .expect("Setup should be able to download bangs.");
            println!("Resolved: {:?}", resolve(search_query.as_str()));
        }
        LaunchType::Validate { verbose } => {
            info!("Reading {}", &args.config.display());
            let mut config_buffer = String::new();
            let mut breader = BufReader::new(File::open(&args.config)?);
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
    };

    Ok(())
}

/// Serve the web server on `address` and `port`
///
/// # Panics
/// Panics if the server could not bind to the desired address/port.
pub async fn serve(address: &str, port: u16) {
    info!(name:"Boom", "Starting Web Server on {}:{}", address, port);

    let router = Router::new()
        .route("/", get(redirector))
        .route("/bangs", get(list_bangs));

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
    axum::serve(listener, router).await.unwrap();
}
