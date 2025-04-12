use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use futures_util::StreamExt;
use ntex::http::Client;

pub async fn grab_remote(url: String, out: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
    // let client = Client::new();
    // let mut res = client.get(url).send().await?;
    // let mut body = res.take_payload();
    //
    // while let Some(chunk) = body.next().await {
    //     let bytes = chunk?;
    //     // Interpret as ASCII or UTF-8 text
    //     let text = str::from_utf8(&bytes)?;
    //     print!("{}", text); // or handle it however you want
    // }
    //
    // Ok(())
}
