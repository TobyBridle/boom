pub mod parse_bangs;
pub mod parse_templates;
pub mod resolver;

use std::{cmp::max, ops::Range};

use serde::Deserialize;

pub static DEFAULT_SEARCH_TEMPLATE: &str = "https://google.com/search?q={{{s}}}";
pub static DEFAULT_SEARCH_INDEXES: std::sync::LazyLock<Match> =
    std::sync::LazyLock::new(|| Match::new(28, DEFAULT_SEARCH_TEMPLATE.len()));

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Match {
    /// Inclusive start index of a match
    pub start: usize,

    /// Exclusive end index of a match
    pub end: usize,
}

impl Match {
    #[inline(always)]
    pub fn new(start: usize, end: usize) -> Self {
        Match { start, end }
    }

    #[inline(always)]
    pub fn is_empty(self) -> bool {
        self.start == 0 && self.end == 0
    }

    #[inline(always)]
    pub fn to_indices(self, offset: usize) -> Range<usize> {
        (max(self.start, offset) - offset)..(max(self.end, offset + 1) - offset)
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
