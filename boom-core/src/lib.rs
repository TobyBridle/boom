use serde::{Deserialize, Serialize};

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
