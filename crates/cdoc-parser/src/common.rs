use cowstr::CowStr;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::ops::Range;

// #[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
// pub struct Span {
//     pub input: CowStr,
//     pub start: usize,
//     pub end: usize,
// }

// impl Span {
//     pub fn new(input: CowStr, start: usize, end: usize) -> Self {
//         Span { input, start, end }
//     }
//
//     pub fn get_with_margin(&self, margin: usize) -> &str {
//         &self.input[self.start.checked_sub(margin).unwrap_or_default()
//             ..min(self.end + margin, self.input.len())]
//     }
// }

// impl<'a> From<pest::Span<'a>> for Span {
//     fn from(value: pest::Span) -> Self {
//         Span {
//             input: value.get_input().into(),
//             start: value.start(),
//             end: value.end(),
//         }
//     }
// }

#[derive(Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
pub struct Span {
    pub range: Range<usize>,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { range: start..end }
    }

    pub fn get_with_margin<'a>(&self, input: &'a str, margin: usize) -> &'a str {
        &input[self.range.start.checked_sub(margin).unwrap_or_default()
            ..min(self.range.end + margin, input.len())]
    }
}

impl From<pest::Span<'_>> for Span {
    fn from(value: pest::Span) -> Self {
        Self {
            range: value.start()..value.end(),
        }
    }
}
