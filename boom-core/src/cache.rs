use std::{
    collections::HashMap,
    error::Error,
    sync::{LazyLock, RwLock, RwLockReadGuard},
};

use tracing::info;

#[cfg(feature = "history")]
use crate::HistoryEntry;
use crate::Redirect;

pub static CACHE: LazyLock<RwLock<HashMap<String, usize>>> =
    LazyLock::new(|| RwLock::new(HashMap::with_capacity(128)));

static REDIRECT_LIST: LazyLock<RwLock<Vec<Redirect>>> = LazyLock::new(|| RwLock::new(vec![]));

#[cfg(feature = "history")]
pub static SEARCH_HISTORY_CACHE: LazyLock<RwLock<Vec<HistoryEntry>>> =
    LazyLock::new(|| RwLock::new(vec![]));

/// Initialises the list of redirects, unless specified otherwise using `overwrite`.
///
/// # Errors
/// If the list already exists AND `overwrite` is false.
/// If a write lock is not acquired on the list.
///
/// # Example
/// ```
/// use boom_core::cache::init_list;
/// use boom_core::Redirect;
///
/// fn get_bangs_from_file() -> Vec<Redirect> { vec![] }
/// let bangs = get_bangs_from_file();
/// init_list(bangs, false).unwrap();
/// ```
pub fn init_list(
    mut redirects: Vec<Redirect>,
    overwrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        if !get_redirects()?.is_empty() && !overwrite {
            return Err("List already initialised".into());
        }
    }
    REDIRECT_LIST.try_write()?.append(&mut redirects);
    Ok(())
}

/// Get an instance of the `REDIRECT_LIST` wrapped within a read guard.
///
/// # Errors
/// This function will error if the `try_read` call fails.
/// Please check the documentation of [`std::sync::poison::rwlock::RwLock::try_read`] for more info
pub fn get_redirects<'a>() -> Result<RwLockReadGuard<'a, Vec<Redirect>>, Box<dyn Error>> {
    match REDIRECT_LIST.try_read() {
        Ok(list) => Ok(list),
        Err(e) => Err(Box::new(e)),
    }
}

/// Set the value of the global `REDIRECT_LIST`.
/// **This does not append, it overwrites.**
///
/// # Errors
/// This function will error if the `try_write` call fails.
/// Please check the documentation of [`std::sync::poison::rwlock::RwLock::try_write`] for more info
pub fn set_redirects(redirects: Vec<Redirect>) -> Result<(), Box<dyn std::error::Error>> {
    (*REDIRECT_LIST.try_write()?) = redirects;
    Ok(())
}

/// Insert (or update) a bang and its index in the list of valid bangs
///
/// # Errors
/// Errors if a write lock is unable to be acquired on `CACHE`.
///
/// # Example
/// ```
/// use boom_core::cache::{insert_bang};
///
/// fn get_index(key: &str) -> Option<usize> {
///     // fancy schmancy key grabbing logic here
///     Some(0) // default value for the sake of an example
/// }
///
/// let i = get_index("yt").unwrap();
/// insert_bang("yt".to_string(), i).ok().unwrap_or_else(|| println!("yt bang does not exist"));
/// ```
pub fn insert_bang(bang: String, template_index: usize) -> Result<(), Box<dyn std::error::Error>> {
    CACHE.try_write()?.insert(bang, template_index);
    Ok(())
}

/// Try to get a bang and its index in the list of valid bangs
///
/// # Errors
/// Errors if a read lock is unable to be acquired on `CACHE`.
///
/// # Example
/// ```
/// use boom_core::cache::get_bang;
///
/// let does_bang_exist = get_bang("yt").unwrap().is_some();
/// ```
pub fn get_bang(bang: &str) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    Ok(CACHE.try_read()?.get(bang).copied())
}

/// Attempt to update a redirect, replacing it if found, and pushing it onto the [`REDIRECT_LIST`]
/// if not found.
///
/// # Errors
/// - if a write lock could not be optained on the [`REDIRECT_LIST`]
/// - if the [`get_bang`] fails
/// - if the bang insertion fails
pub fn update_redirect(redirect: &Redirect) -> Result<(), Box<dyn std::error::Error>> {
    let mut write_lock = REDIRECT_LIST
        .write()
        .map_err(|e| format!("RwLock poisoned: {e}"))?;

    if let Some(idx) = get_bang(&redirect.trigger)? {
        info!("Replacing `!{}` in cache", redirect.trigger);
        write_lock[idx] = redirect.clone();
    } else {
        write_lock.push(redirect.clone());
        insert_bang(redirect.trigger.clone(), write_lock.len() - 1)
            .map_err(|e| format!("Insert bang failed: {e}"))?;
    }
    drop(write_lock);

    Ok(())
}

/// Set the value of the global `SEARCH_HISTORY_CACHE`.
/// **This does not append, it overwrites.**
///
/// # Errors
/// This function will error if the `try_write` call fails.
/// Please check the documentation of [`std::sync::poison::rwlock::RwLock::try_write`] for more info
#[cfg(feature = "history")]
pub fn set_history_queries(queries: &[HistoryEntry]) -> Result<(), Box<dyn std::error::Error>> {
    (*SEARCH_HISTORY_CACHE.try_write()?) = queries.to_vec();
    Ok(())
}

/// Pushes onto the vector [`SEARCH_HISTORY_CACHE`].
///
/// # Errors
/// This function will error if the `try_write` call fails.
/// Please check the documentation of [`std::sync::poison::rwlock::RwLock::try_write`] for more info
#[cfg(feature = "history")]
pub fn add_history_query(query: HistoryEntry) -> Result<(), Box<dyn std::error::Error>> {
    SEARCH_HISTORY_CACHE.try_write()?.push(query);
    Ok(())
}
