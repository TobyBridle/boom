//! # About
//!
//! `boom-core` provides the integral parts of `boom`.
//! It provides functions for efficiently extracting data from queries and templates,
//! as well as higher-level functions such as `resolve`

use std::{cmp::Ordering, time::Duration};

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

const SOURCE_IDENTIFIER_EMPTY: &str = "n/a";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SourceIdentifier {
    Identifier(String),

    #[default]
    Empty,
}

impl From<String> for SourceIdentifier {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            SOURCE_IDENTIFIER_EMPTY => Self::Empty,
            _ => Self::Identifier(value),
        }
    }
}

impl From<SourceIdentifier> for String {
    fn from(value: SourceIdentifier) -> Self {
        match value {
            SourceIdentifier::Identifier(si) => si,
            SourceIdentifier::Empty => SOURCE_IDENTIFIER_EMPTY.to_string(),
        }
    }
}

impl PartialOrd for SourceIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourceIdentifier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (&self, &other) {
            (Self::Identifier(a), Self::Identifier(b)) => a.cmp(b),
            (Self::Identifier(_), Self::Empty) => Ordering::Greater,
            (Self::Empty, Self::Identifier(_)) => Ordering::Less,
            (Self::Empty, Self::Empty) => Ordering::Equal,
        }
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        match (&self, &other) {
            (Self::Empty, Self::Empty) => Self::Empty,
            (Self::Identifier(a), Self::Identifier(b)) => {
                if matches!(a.cmp(b), Ordering::Greater | Ordering::Equal) {
                    self
                } else {
                    other
                }
            }
            (Self::Identifier(_), Self::Empty) => self,
            (Self::Empty, Self::Identifier(_)) => other,
        }
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        match (&self, &other) {
            (Self::Identifier(a), Self::Identifier(b)) => {
                if a.min(b) == a {
                    self
                } else {
                    other
                }
            }
            (Self::Identifier(_), Self::Empty) => other,
            (Self::Empty, Self::Identifier(_) | Self::Empty) => self,
        }
    }
}

impl Serialize for SourceIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SourceIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            SOURCE_IDENTIFIER_EMPTY => Self::Empty,
            _ => Self::Identifier(s),
        })
    }
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

pub type EncodedQuery = (String /* bang */, String /* query */);

#[cfg(feature = "history")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub query: EncodedQuery,
    pub timestamp: i64,
    pub source_identifier: SourceIdentifier,
}
