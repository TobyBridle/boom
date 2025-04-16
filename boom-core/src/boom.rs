pub mod grab_remote_bangs;
pub mod parse_bangs;
pub mod parse_templates;
pub mod resolver;

use std::{cmp::max, ops::Range};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Match {
    /// Inclusive start index of a match
    pub start: usize,

    /// Exclusive end index of a match
    pub end: usize,
}

impl Match {
    #[inline]
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == 0 && self.end == 0
    }

    #[inline]
    #[must_use]
    pub fn to_indices(self, offset: usize) -> Range<usize> {
        (max(self.start, offset) - offset)..(max(self.end, offset + 1) - offset)
    }
}
