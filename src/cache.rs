use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use crate::boom::Redirect;

pub static CACHE: LazyLock<RwLock<HashMap<String, usize>>> =
    LazyLock::new(|| RwLock::new(HashMap::with_capacity(128)));

pub static REDIRECT_LIST: LazyLock<RwLock<Vec<Redirect>>> = LazyLock::new(|| RwLock::new(vec![]));

/// Initialises the list of redirects, unless specified otherwise using `overwrite`.
///
/// # Errors
/// If the list already exists AND `overwrite` is false.
/// If a write lock is not acquired on the list.
///
/// # Example
/// ```
/// let bangs = get_bangs_from_file()?;
/// init_list(bangs, false)?;
/// ```
pub fn init_list(
    mut redirects: Vec<Redirect>,
    overwrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        if !REDIRECT_LIST.try_read()?.is_empty() && !overwrite {
            return Err("List already initialised".into());
        };
    }
    REDIRECT_LIST.try_write()?.append(&mut redirects);
    Ok(())
}

/// Insert (or update) a bang and its index in the list of valid bangs
///
/// # Errors
/// Errors if a write lock is unable to be acquired on `CACHE`.
///
/// # Example
/// ```
/// // https://google.com/search?q={{{s}}}
/// //                             ^     ^
///                               <x>   <y>
/// let z: usize = get_index("yt");
/// insert_bang("yt".to_string(), x, y, z).ok()?;
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
/// let does_bang_exist = get_bang("yt")?.is_some();
/// ```
pub fn get_bang(bang: &str) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    Ok(CACHE.try_read()?.get(bang).copied())
}
