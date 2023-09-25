//! Types for exercise definitions.

use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};

use cowstr::CowStr;
use std::io::{BufWriter, Write};

pub trait Output {
    fn write_string(&self, solution: bool) -> String;
}

/// Represents a line of source code. Can either be markup (descriptions of the exercise) or
/// code (regular source code).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Content {
    Markup(CowStr),
    Src(CowStr),
}

/// An exercise element with a placeholder and a solution
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Solution {
    pub placeholder: Option<CowStr>,
    pub solution: CowStr,
}

/// Top-level structure. A code file is split into these types.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "value", untagged)]
pub enum CodeElem {
    Solution(Solution),
    Src(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct CodeContent {
    pub blocks: Vec<CodeElem>,
    pub meta: LinkedHashMap<CowStr, CowStr>,
    pub hash: u64,
}

impl CodeContent {
    pub fn to_string(&self, with_solution: bool) -> anyhow::Result<String> {
        let mut buf = BufWriter::new(Vec::new());
        for block in &self.blocks {
            match block {
                CodeElem::Solution(s) => {
                    if with_solution {
                        buf.write_all(s.solution.as_bytes())?;
                    } else {
                        s.placeholder
                            .as_ref()
                            .map(|p| buf.write(p.as_bytes()))
                            .transpose()?;
                    }
                }
                CodeElem::Src(s) => {
                    buf.write_all(s.as_bytes())?;
                }
            }
        }

        Ok(String::from_utf8(buf.into_inner()?)?)
    }
}
