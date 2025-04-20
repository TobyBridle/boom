use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

pub mod boom;
pub mod cache;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Redirect {
    /// The short name or abbreviation of the bang command.
    #[serde(rename = "s")]
    pub short_name: String,
    /// The trigger text for the bang command (e.g., "g" for Google).
    #[serde(rename = "t")]
    pub trigger: String,
    /// The URL template where the search term is inserted.
    #[serde(rename = "u")]
    pub url_template: String,
}

async fn has_internet(client: &Client) -> bool {
    let req = client
        .get("http://clients3.google.com/generate_204")
        .build()
        .expect("Request should be valid");
    client
        .execute(req)
        .await
        .map(|res| res.status().is_success())
        .unwrap_or(false)
}

pub async fn await_internet() {
    let client = Client::new();
    while !has_internet(&client).await {
        info!("Waiting for internet to be available.");
        std::thread::sleep(Duration::from_secs(5));
    }
}
