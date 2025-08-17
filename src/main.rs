//! Boom relies on DuckDuckGo-style "bangs" to enable quicker searches
//!
//! ## Crates in use:
//! - [`boom_config`]
//! - [`boom_core`]
//! - [`boom_web`]
//!
//! ## Performance
//! [`boom_core::boom::resolver::resolve`] is capable of resolving queries in just a couple of
//! microseconds.
//! Though overkill, it is able to use SIMD to parse gigantic queries without a
//! struggle.
//!
//! From a cold start, the boom executable is able to read the user config, fetch (catched)
//! sources, and resolve a query, in less than 10ms.
//! Considering this was benchmarked using the default `DuckDuckGo` bangs, a JSON file containing over 13,000 unique bangs, 10ms is quite an impressive
//! number.
//!
//! ## Development
//! A test-driven development approach, combined with constant benchmarking, allows boom to be
//! very performant, whilst being ready for edge-cases.

use std::{
    io,
    process::exit,
    sync::{Arc, RwLock},
};

use boom_config::{ConfigBuilder, ConfigSource};
use boom_core::boom::{resolver::resolve, update_bangs_from_config::update_bangs_from_config};
use boom_web::serve;
use clap::Parser;
use cli::{LaunchType, SetupMode};
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

    let args = Arc::new(cli::Args::parse());

    if let LaunchType::Validate { verbose } = args.launch {
        info!("Reading {}", &args.config.display());

        match &args.config.read_into_builder() {
            Ok(cfg) => {
                dbg!(cfg.clone().build());
                info!("Parsed config with no errors.");
            }
            Err(e) => error!("{}", e),
        }

        if verbose {
            info!("Verbose mode is enabled");
        }

        exit(1);
    }

    let config = &args
        .config
        .read_into_builder()
        .unwrap_or_else(|e| {
            eprintln!("Could not read Config. Reason: {e:?}");
            eprintln!("Falling back to default config.");
            ConfigBuilder::default()
        })
        .add_source(args.as_ref())
        .to_owned()
        .build();

    let setup = args.launch.setup_type();

    update_bangs_from_config(
        Arc::new(config.bangs.clone()),
        Arc::new(RwLock::new(vec![])),
        matches!(setup, SetupMode::Caches),
        false,
    )
    .await;

    #[allow(clippy::match_wildcard_for_single_variants)]
    match &args.launch {
        LaunchType::Serve {
            addr,
            port,
            await_internet,
        } => {
            if *await_internet {
                boom_core::await_internet().await;
            }

            serve(*addr, *port, config).await;
        }
        LaunchType::Resolve { search_query, .. } => {
            println!("Resolved: {:?}", resolve(search_query.as_str(), config));
        }
        _ => {}
    }

    Ok(())
}
