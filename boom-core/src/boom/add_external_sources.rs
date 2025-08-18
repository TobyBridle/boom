use std::{
    process::exit,
    sync::{Arc, RwLock},
};

use boom_config::BangSourceConfig;
use tokio::task::JoinSet;
use tracing::{error, info, warn};

use crate::{
    Redirect,
    boom::{grab_remote_bangs::download_remote, parse_bangs::parse_bang_file},
};
use expanduser::expanduser;

/// Add a list of bangs into a slice of existing Bangs
/// Attempts to optimise execution time by only downloading remote sources
/// when the cached resource is explicitly ignored.
///
/// > **NOTE**: This function may error, without causing a [`panic!`] or exiting the process.
/// > Error/warning logs will be produced, though the program will continue as usual, if the source
/// > was not required.
pub async fn add_external_sources(
    sources: Arc<&[BangSourceConfig]>,
    bangs: Arc<RwLock<Vec<Redirect>>>,
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

            let filepath = source
                .filepath
                .to_str()
                .ok_or_else(|| format!("Could not convert {} into str", source.filepath.display()))
                .map_or_else(
                    |filepath_str| {
                        expanduser(&filepath_str).unwrap_or_else(|_| source.filepath.clone())
                    },
                    |_| source.filepath.clone(),
                );

            match parse_bang_file(&filepath) {
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
        let Ok(mut lock) = bangs.write() else {
            error!("Could not acquire write lock on bangs.");
            continue;
        };
        lock.extend(res.unwrap_or_else(|_| {
            warn!("Unable to get Redirects from JoinSet");
            vec![]
        }));
    }
}
