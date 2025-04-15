use std::io;

use boom_config::read_config::read_config;
use boom_core::boom::resolver::resolve;
use boom_web::serve;
use clap::Parser;
use cli::LaunchType;
use tracing::{Level, error, info};
pub mod cli;

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
    // let config = read_config(&args.config).expect("Config path should be valid & readable.");

    match args.launch {
        LaunchType::Serve { addr, port } => serve(addr.as_str(), port).await,
        LaunchType::Resolve { search_query } => {
            println!("Resolved: {:?}", resolve(search_query.as_str()));
        }
        LaunchType::Validate { verbose } => {
            info!("Reading {}", &args.config.display());
            match read_config(&args.config) {
                Ok(cfg) => {
                    if verbose {
                        dbg!(cfg);
                    }
                    info!("Parsed config with no errors.");
                }
                Err(e) => error!(e),
            }
        }
    }

    Ok(())
}
