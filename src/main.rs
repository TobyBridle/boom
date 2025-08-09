use std::{io, process::exit, sync::Arc};

use boom_config::{ConfigBuilder, ConfigSource};
use boom_core::{
    Redirect,
    boom::{grab_remote_bangs::download_remote, parse_bangs::parse_bang_file, resolver::resolve},
    cache::{insert_bang, set_redirects},
};
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

    let mut bangs = if config.bangs.default.enabled {
        if matches!(setup, SetupMode::All) || !config.bangs.default.filepath.try_exists()? {
            download_remote(&config.bangs.default.remote, &config.bangs.default.filepath)
                .await
                .unwrap_or_else(|e| eprintln!("Could not fetch bangs from remote ({}). Continuing without default bangs.\nError: {e:?}", &config.bangs.default.remote));
        }

        parse_bang_file(&config.bangs.default.filepath).unwrap_or_else(|e| {
            eprintln!("Could not read bang file. Error: {e:?}");
            exit(1);
        })
    } else {
        vec![]
    };

    config.bangs.custom.iter().for_each(|(short_name, custom)| {
        bangs.push(Redirect {
            short_name: short_name.clone(),
            trigger: custom.trigger.clone(),
            url_template: custom.template.clone(),
        });
    });
    bangs.iter().enumerate().for_each(|(i, r)| {
        insert_bang(r.trigger.clone(), i).unwrap_or_else(|_| {
            eprintln!(
                "Bang ({}) should not already exist within the cache",
                r.trigger
            );
        });
    });
    set_redirects(bangs).unwrap();

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
        LaunchType::Validate { verbose } => {
            info!("Reading {}", &args.config.display());
            match &args.config.read_into_builder() {
                Ok(cfg) => {
                    if *verbose {
                        dbg!(cfg.clone().build());
                    }
                    info!("Parsed config with no errors.");
                }
                Err(e) => error!(e),
            }
        }
    }

    Ok(())
}
