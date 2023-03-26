mod collectors;
mod events;
mod iterators;
mod visitor;

pub use collectors::*;
pub use events::*;
pub use visitor::*;

use crate::notebook::CellOutput;
use pulldown_cmark::{HeadingLevel, LinkType};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Inline {
    Text(String),
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    SoftBreak,
    HardBreak,
    Rule,
    Image(LinkType, String, String, Vec<Inline>),
    Link(LinkType, String, String, Vec<Inline>),
    Html(String),
}

fn vec_inline_to_string(vec: &[Inline]) -> String {
    vec.iter().map(|item| item.to_string()).collect()
}

impl ToString for Inline {
    fn to_string(&self) -> String {
        match self {
            Inline::Text(s) => s.clone(),
            Inline::Emphasis(inner) => vec_inline_to_string(inner),
            Inline::Strong(inner) => vec_inline_to_string(inner),
            Inline::Strikethrough(inner) => vec_inline_to_string(inner),
            Inline::Code(s) => s.clone(),
            Inline::SoftBreak => String::default(),
            Inline::HardBreak => String::default(),
            Inline::Rule => String::default(),
            Inline::Html(s) => s.to_string(),
            _ => String::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ast(pub(crate) Vec<Block>);

#[allow(unused)]
#[derive(Clone, Debug, Default)]
pub struct CodeAttributes {
    pub(crate) editable: bool,
    pub(crate) fold: bool,
}

#[derive(Clone, Debug)]
pub enum CodeOutput {
    Image(String),
    Svg(String),
    Json(HashMap<String, Value>),
    Html(String),
    Javascript(String),
}

#[derive(Clone, Debug)]
pub enum Block {
    Heading {
        lvl: HeadingLevel,
        id: Option<String>,
        classes: Vec<String>,
        inner: Vec<Inline>,
    },
    Plain(Inline),
    Paragraph(Vec<Inline>),
    BlockQuote(Vec<Inline>),
    CodeBlock {
        source: String,
        reference: Option<String>,
        attr: CodeAttributes,
        outputs: Vec<CellOutput>,
    },
    List(Option<u64>, Vec<Block>),
    ListItem(Vec<Block>),
}
