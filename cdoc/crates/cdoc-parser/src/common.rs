use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct PosInfo {
    pub input: String,
    pub start: usize,
    pub end: usize,
}

impl PosInfo {
    pub fn new(input: &str, start: usize, end: usize) -> Self {
        PosInfo {
            input: input.to_string(),
            start,
            end,
        }
    }

    pub fn get_with_margin(&self, margin: usize) -> &str {
        &self.input[max(self.start - margin, 0)..min(self.end + margin, self.input.len())]
    }
}

impl<'a> From<pest::Span<'a>> for PosInfo {
    fn from(value: pest::Span) -> Self {
        PosInfo {
            input: value.get_input().to_string(),
            start: value.start(),
            end: value.end(),
        }
    }
}
