use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
};

use super::{SyncAdd, SyncHandler};
use crate::{
    boom::Redirect,
    cache::{get_redirects, insert_bang, set_redirects},
};
use boom::cache::CACHE;
use tracing::info;

impl SyncAdd for SyncHandler {
    fn add_bang(&mut self, r: Redirect) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(_existent_bang) = CACHE.try_read()?.get(&r.trigger) {
            return Err(Box::from(format!(
                "Could not add {r:?}. Trigger already exists in the cache."
            )));
        }

        let mut redirect_list_read = get_redirects()?.clone();
        let new_len = redirect_list_read.len().max(1) - 1;

        let mut bang_writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(self.bang_path.get_mut())?,
        );

        redirect_list_read.push(r.clone());
        serde_json::to_writer(&mut bang_writer, &redirect_list_read)?;

        set_redirects(redirect_list_read)?;
        bang_writer.flush()?;

        insert_bang(r.trigger, new_len)?;
        info!("Inserted bang into the cache.");

        Ok(())
    }
}
