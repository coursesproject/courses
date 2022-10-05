use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub trait Output {
    fn to_string(&self, solution: bool) -> String;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "content", untagged)]
pub enum Content {
    #[serde(rename = "markup")]
    Markup { markup: String },
    #[serde(rename = "code")]
    Code { code: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "code_block")]
pub struct SolutionBlock {
    pub placeholder: Vec<Content>,
    pub solution: Vec<Content>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "inner", untagged)]
pub enum Inner {
    #[serde(rename = "code_block")]
    SolutionBlock(SolutionBlock),
    #[serde(rename = "src")]
    SrcBlock(Content),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "block")]
pub struct Block {
    pub keyword: String,
    pub attributes: HashMap<String, String>,
    pub inner: Vec<Inner>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "value", untagged)]
pub enum Value {
    #[serde(rename = "block")]
    Block { block: Block },
    #[serde(rename = "src")]
    SrcBlock { content: Content },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "document")]
pub struct Document {
    pub blocks: Vec<Value>,
}

impl Document {
    fn to_json(&self) -> String {
        serde_json::to_string(&self).expect("Could not construct JSON representation.")
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl<T> Output for Vec<T>
where
    T: Output,
{
    fn to_string(&self, solution: bool) -> String {
        self.into_iter()
            .map(|v| v.to_string(solution))
            .collect::<Vec<String>>()
            .join("")
    }
}

impl Output for Content {
    fn to_string(&self, _: bool) -> String {
        match self {
            Content::Code { code: value } => value.to_string(),
            Content::Markup { markup: _value } => "".to_string(),
        }
    }
}

impl Output for Inner {
    fn to_string(&self, solution: bool) -> String {
        match self {
            Inner::SolutionBlock(SolutionBlock{
                placeholder,
                solution: solution_block,
            }) => {
                if solution {
                    solution_block.to_string(solution)
                } else {
                    placeholder.to_string(solution)
                }
            }
            Inner::SrcBlock(content) => content.to_string(solution),
        }
    }
}

impl Output for Block {
    fn to_string(&self, solution: bool) -> String {
        self.inner.to_string(solution)
    }
}

impl Output for Value {
    fn to_string(&self, solution: bool) -> String {
        match self {
            Value::Block { block } => block.to_string(solution),
            Value::SrcBlock { content } => content.to_string(solution),
        }
    }
}

impl Output for Document {
    fn to_string(&self, solution: bool) -> String {
        self.blocks.to_string(solution)
    }
}
