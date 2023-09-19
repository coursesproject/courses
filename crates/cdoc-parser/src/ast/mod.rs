pub mod parser;
pub mod visitor;

use crate::code_ast::types::CodeContent;
use crate::common::PosInfo;
use pulldown_cmark::LinkType;
use serde::{Deserialize, Serialize};

use crate::raw::CodeAttr;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Ast(pub Vec<Block>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Command {
    pub function: String,
    pub label: Option<String>,
    pub parameters: Vec<Parameter>,
    pub body: Option<Vec<Block>>,
    pub pos: PosInfo,
    pub global_idx: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CodeBlock {
    /// Label
    pub label: Option<String>,
    /// Code source
    pub source: CodeContent,
    /// Code tags
    pub attributes: Vec<String>,
    /// Display the block as a cell or listing (only used for notebooks)
    pub display_cell: bool,
    pub global_idx: usize,
    pub pos: PosInfo,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Math {
    pub source: String,
    pub label: Option<String>,
    pub display_block: bool,
    pub pos: PosInfo,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Inline {
    /// Plain text
    Text(String),
    Styled(Vec<Inline>, Style),
    /// Inline code
    Code(String),
    /// A code block. May originate from markdown fenced code blocks or notebook code cells.
    CodeBlock(CodeBlock),
    SoftBreak,
    HardBreak,
    /// Horizontal rule
    Rule,
    /// An inline image (usually originates from a markdown image spec)
    Image(LinkType, String, String, Vec<Inline>),
    /// An inline link (usually originates from a markdown link spec)
    Link(LinkType, String, String, Vec<Inline>),
    /// Unescaped html.
    Html(String),
    /// Math element (may be inline or display)
    /// The trailing space element is necessary due to the way parsing currently works with
    /// pulldown_cmark.
    Math(Math),
    Command(Command),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub key: Option<String>,
    pub value: Value,
    pub pos: PosInfo,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub obj_type: String,
    pub attr: LinkedHashMap<String, String>,
    pub num: usize,
}

// #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// #[serde(tag = "type", rename_all = "lowercase")]
// pub enum Reference {
//     Math {
//         display_inline: bool,
//     },
//     Code {
//         tags: Vec<CodeAttr>,
//     },
//     Command {
//         function: String,
//         parameters: HashMap<String, String>,
//     },
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Flag(String),
    Content(Vec<Block>),
    String(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Block {
    Heading {
        lvl: u8,
        id: Option<String>,
        classes: Vec<String>,
        inner: Vec<Inline>,
    },
    Plain(Vec<Inline>),
    Paragraph(Vec<Inline>),
    BlockQuote(Vec<Inline>),
    /// A list - ordered or unordered.
    List(Option<u64>, Vec<Block>),
    ListItem(Vec<Block>),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CodeMeta {
    pub id: String,
    pub editable: bool,
    pub folded: bool,
    pub custom: LinkedHashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Style {
    Emphasis,
    Strong,
    Strikethrough,
    Underline,
}
