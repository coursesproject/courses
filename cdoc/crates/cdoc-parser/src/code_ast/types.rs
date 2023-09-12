//! Types for exercise definitions.

use crate::ast::CodeMeta;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait Output {
    fn write_string(&self, solution: bool) -> String;
}

/// Represents a line of source code. Can either be markup (descriptions of the exercise) or
/// code (regular source code).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Content {
    Markup(String),
    Src(String),
}

/// An exercise element with a placeholder and a solution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Solution {
    pub placeholder: Option<String>,
    pub solution: String,
}

/// Top-level structure. A code file is split into these types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "value", untagged)]
pub enum CodeBlock {
    Solution(Solution),
    Src(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct CodeContent {
    pub blocks: Vec<CodeBlock>,
    pub meta: HashMap<String, String>,
}
