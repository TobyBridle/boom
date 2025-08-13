use std::{io, process::exit, sync::Arc};

use boom_config::{BangSourceConfig, ConfigBuilder, ConfigSource};
use boom_core::{
    Redirect,
    boom::{grab_remote_bangs::download_remote, parse_bangs::parse_bang_file, resolver::resolve},
    cache::{insert_bang, set_redirects},
};
use boom_web::serve;
use clap::Parser;
use cli::{LaunchType, SetupMode};
use expanduser::expanduser;
use tokio::task::JoinSet;
use tracing::{Level, error, info, warn};
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

    let mut bangs = vec![];

    add_external_sources(
        &config.bangs.sources,
        &mut bangs,
        matches!(setup, SetupMode::Caches),
    )
    .await;

    if bangs.is_empty() {
        warn!("No bangs were loaded. Is this intended?");
    }

    let custom_bangs = config
        .bangs
        .custom
        .iter()
        .map(|(short_name, custom)| Redirect {
            short_name: short_name.clone(),
            trigger: custom.trigger.clone(),
            url_template: custom.template.clone(),
        });

    info!("Loaded {} bangs from config file.", custom_bangs.len());
    bangs.extend(custom_bangs);

    bangs.iter().enumerate().for_each(|(i, r)| {
        insert_bang(r.trigger.clone(), i).unwrap_or_else(|_| {
            eprintln!(
                "Bang ({}) should not already exist within the cache",
                r.trigger
            );
        });
    });
    set_redirects(bangs).unwrap();

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

async fn add_external_sources(
    sources: &[BangSourceConfig],
    bangs: &mut Vec<Redirect>,
    use_cache: bool,
) {
    let mut set = JoinSet::new();

    for source in sources.iter().cloned() {
        set.spawn(async move {
            if !use_cache && let Some(remote) = &source.remote {
                match download_remote(remote, &source.filepath).await {
                    Ok(()) => info!("Fetched bangs from {source}"),
                    Err(e) => {
                        if source.required {
                            error!(
                                "Could not fetch bangs from remote source {source}. Error: {e:?}"
                            );
                            exit(1);
                        } else {
                            warn!(
                                "Could not fetch bangs from remote source {source}. Error: {e:?}"
                            );
                        }
                    }
                }
            }

            let filepath =
                if let Ok(filepath_str) = &source.filepath.to_str().ok_or_else(|| {
                    format!("Could not convert {} into str", source.filepath.display())
                }) && let Ok(p) = expanduser(filepath_str)
                {
                    &p.clone()
                } else {
                    warn!("Could not expand {source}. Continuing with path as is.");
                    &source.filepath
                };

            match parse_bang_file(filepath) {
                Ok(bangs) => {
                    info!("Loaded {} bangs from source {}", bangs.len(), source);
                    bangs
                }
                Err(e) => {
                    if source.required {
                        error!("Could not read bang source {source}. Error: {e:?}");
                        exit(1);
                    } else {
                        warn!("Skipping bang source {source}. Error: {e:?}");
                        vec![]
                    }
                }
            }
        });
    }

    while let Some(res) = set.join_next().await {
        bangs.extend(res.unwrap());
    }
}
