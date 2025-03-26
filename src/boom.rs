pub mod parse_bangs;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Match {
    /// Inclusive start index of a match
    pub start: usize,

    /// Exclusive end index of a match
    pub end: usize,
}

impl Match {
    pub fn new(start: usize, end: usize) -> Self {
        Match { start, end }
    }

    pub fn is_empty(self) -> bool {
        self.start == 0 && self.end == 0
    }
}

#[derive(Deserialize, Debug)]
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
