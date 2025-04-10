use std::{cell::Cell, path::PathBuf};

use crate::boom::Redirect;

pub mod add_bang;

pub struct SyncHandler {
    bang_path: Cell<PathBuf>,
}

impl SyncHandler {
    #[must_use]
    pub const fn new(bang_path: PathBuf) -> Self {
        Self {
            bang_path: Cell::new(bang_path),
        }
    }
}

pub trait SyncAdd {
    /// Add a bang and its corresponding template into the cache,
    /// syncing it to disk at `~/.config/boom/default_bangs.json`.
    ///
    /// # Errors
    /// - Errors if the trigger is already found in the cache.
    /// - The bang file could not be opened with write access
    /// - The global redirect list could not be read with `get_redirects`
    /// - The redirect list could not be serialized to json
    /// - The contents of the underlying writer could not be flushed
    fn add_bang(&mut self, r: Redirect) -> Result<(), Box<dyn std::error::Error>>;
}
