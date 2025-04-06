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
/// use boom::cache::init_list;
/// use boom::boom::Redirect;
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
        if !REDIRECT_LIST.try_read()?.is_empty() && !overwrite {
            return Err("List already initialised".into());
        }
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
/// use boom::cache::{insert_bang};
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
/// use boom::cache::get_bang;
///
/// let does_bang_exist = get_bang("yt").unwrap().is_some();
/// ```
pub fn get_bang(bang: &str) -> Result<Option<usize>, Box<dyn std::error::Error>> {
    Ok(CACHE.try_read()?.get(bang).copied())
}
