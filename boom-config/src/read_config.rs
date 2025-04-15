use std::{
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

use tracing::info;

use super::{Config, parse_config::parse_config};

/// Reads (& parses) a config file from `config_path`, attempting to create a default one if it does not
/// exist.
///
/// # Errors
/// If the default config file cannot be created/copied to
/// If the config cannot be opened
/// If the contents of the config file are not valid UTF-8
pub fn read_config(config_path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    if !config_path.exists() {
        info!("Creating default config file at {config_path:?}");
        let source_path = Path::new("src/config/default_config.toml");
        fs::copy(source_path, config_path)?;
    }

    let mut reader = BufReader::new(File::open(config_path)?);
    let mut buffer = String::with_capacity(4096);
    reader.read_to_string(&mut buffer)?;

    Ok(parse_config(&buffer))
}
