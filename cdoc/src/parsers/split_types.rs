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
#[serde(rename = "content", untagged)]
pub enum Content {
    #[serde(rename = "markup")]
    Markup { markup: String },
    #[serde(rename = "code")]
    Code { code: String },
}

/// An exercise element with a placeholder and a solution
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "code_block")]
pub struct ExerciseBlock {
    pub placeholder: Option<Vec<Content>>,
    pub solution: Vec<Content>,
}

/// A task consists of exercise definitions or regular source code.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "inner", untagged)]
pub enum Inner {
    #[serde(rename = "code_block")]
    ExerciseBlock(ExerciseBlock),
    #[serde(rename = "src")]
    SrcBlock(Content),
}

/// Describes a block. Currently, only Code blocks are available.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "block")]
pub struct Block {
    pub keyword: String,
    pub attributes: HashMap<String, String>,
    pub inner: Vec<Inner>,
}

/// Top-level structure. A code file is split into these types.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "value", untagged)]
pub enum SplitParseValue {
    #[serde(rename = "block")]
    Block { block: Block },
    #[serde(rename = "src")]
    SrcBlock { content: Content },
    #[serde(rename = "code_block")]
    SolutionBlock(ExerciseBlock),
    #[serde(rename = "meta")]
    Meta(String, String),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename = "document")]
pub struct CodeTaskDefinition {
    pub blocks: Vec<SplitParseValue>,
}

impl CodeTaskDefinition {
    #[allow(unused)]
    fn to_json(&self) -> String {
        serde_json::to_string(&self).expect("Could not construct JSON representation.")
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl From<CodeTaskDefinition> for CodeMeta {
    fn from(value: CodeTaskDefinition) -> Self {
        let mut tmp = CodeMeta::default();

        for v in value.blocks {
            if let SplitParseValue::Meta(k, v) = v {
                tmp.custom.insert(k, v);
            }
        }

        tmp
    }
}

impl<T> Output for Vec<T>
where
    T: Output,
{
    fn write_string(&self, solution: bool) -> String {
        self.iter()
            .map(|v| v.write_string(solution))
            .collect::<Vec<String>>()
            .join("")
    }
}

impl Output for Content {
    fn write_string(&self, _: bool) -> String {
        match self {
            Content::Code { code: value } => value.to_string(),
            Content::Markup { markup: _value } => "".to_string(),
        }
    }
}

impl Output for Inner {
    fn write_string(&self, solution: bool) -> String {
        match self {
            Inner::ExerciseBlock(ExerciseBlock {
                placeholder,
                solution: solution_block,
            }) => {
                if solution {
                    solution_block.write_string(solution)
                } else {
                    placeholder
                        .as_ref()
                        .map(|p| p.write_string(solution))
                        .unwrap_or_default()
                }
            }
            Inner::SrcBlock(content) => content.write_string(solution),
        }
    }
}

impl Output for Block {
    fn write_string(&self, solution: bool) -> String {
        self.inner.write_string(solution)
    }
}

impl Output for SplitParseValue {
    fn write_string(&self, solution: bool) -> String {
        match self {
            SplitParseValue::Block { block } => block.write_string(solution),
            SplitParseValue::SrcBlock { content } => content.write_string(solution),
            SplitParseValue::SolutionBlock(ExerciseBlock {
                placeholder,
                solution: solution_block,
            }) => {
                if solution {
                    solution_block.write_string(solution)
                } else {
                    placeholder
                        .as_ref()
                        .map(|p| p.write_string(solution))
                        .unwrap_or_default()
                }
            }
            SplitParseValue::Meta(_, _) => String::default(),
        }
    }
}

impl Output for CodeTaskDefinition {
    fn write_string(&self, solution: bool) -> String {
        self.blocks.write_string(solution)
    }
}

impl CodeTaskDefinition {
    pub fn split(&self) -> (String, String) {
        (self.write_string(false), self.write_string(true))
    }
}
