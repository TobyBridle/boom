use std::sync::{Arc, RwLock};

use boom_config::BangConfig;
use tracing::{error, info, warn};

use crate::{
    Redirect,
    boom::add_external_sources::add_external_sources,
    cache::{insert_bang, set_redirects, update_redirect},
};

/// Updates internal caches for bangs from the given configuration
///
/// If `overwrite` is specified, [`update_redirect`] will be used, otherwise, if `overwrite` is
/// false, each bang will attempt to be inserted using [`insert_bang`] and [`set_redirects`]
pub async fn update_bangs_from_config(
    config_bangs: Arc<BangConfig>,
    bangs: Arc<RwLock<Vec<Redirect>>>,
    use_cache: bool,
    overwrite: bool,
) {
    add_external_sources(
        Arc::new(&config_bangs.sources),
        Arc::clone(&bangs),
        use_cache,
    )
    .await;

    if let Ok(rlock) = &bangs.try_read()
        && rlock.is_empty()
    {
        warn!("No bangs were loaded. Is this intended?");
    }

    let custom_bangs = config_bangs
        .custom
        .iter()
        .map(|(short_name, custom)| Redirect {
            short_name: short_name.clone(),
            trigger: custom.trigger.clone(),
            url_template: custom.template.clone(),
        });

    info!("Loaded {} bangs from config file.", custom_bangs.len());

    if let Ok(mut wlock) = bangs.try_write() {
        wlock.extend(custom_bangs);

        if overwrite {
            wlock.iter().for_each(|r| {
                update_redirect(r).unwrap_or_else(|_| warn!("Could not update `!{}`", r.trigger));
            });
        } else {
            wlock.iter().enumerate().for_each(|(i, r)| {
                insert_bang(r.trigger.clone(), i).unwrap_or_else(|_| {
                    warn!(
                        "Bang ({}) should not already exist within the cache",
                        r.trigger
                    );
                });
            });
            set_redirects(wlock.to_vec()).unwrap_or_else(|_| error!("Could not write redirects."));
        }
    }
}
