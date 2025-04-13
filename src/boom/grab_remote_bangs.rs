use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::PathBuf,
};

use reqwest::Client;

pub async fn grab_remote(url: String, out: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut res = client.get(url).send().await?;

    let mut writer = BufWriter::new(OpenOptions::new().write(true).truncate(true).open(out)?);
    while let Some(chunk) = res.chunk().await? {
        let _ = writer.write(&chunk)?;
    }

    Ok(())
}
