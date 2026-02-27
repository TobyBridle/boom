use std::{fs, path::PathBuf};

use tracing::info;

use crate::{Assets, ConfigBuilder, ConfigSource};

/// Reads (& parses) a config file from `config_path`, attempting to create a default one if it does not
/// exist.
///
/// # Errors
/// If the default config file cannot be created/copied to
/// If the config cannot be opened
/// If the contents of the config file are not valid UTF-8
impl ConfigSource for PathBuf {
    fn read_into_builder(&self) -> Result<ConfigBuilder, Box<dyn std::error::Error>> {
        if !self.exists() {
            info!("Creating default config file at {:?}", &self);
            if let Some(parent_dir) = self.parent() {
                fs::create_dir_all(parent_dir)?;
            }
            fs::write(
                self,
                Assets::get("default_config.toml")
                    .ok_or("Expected default config to exist")?
                    .data,
            )?;
        }

        Ok(toml::de::from_str::<ConfigBuilder>(&fs::read_to_string(
            self,
        )?)?)
    }

    fn source(&self) -> Option<&PathBuf> {
        Some(self)
    }
}
