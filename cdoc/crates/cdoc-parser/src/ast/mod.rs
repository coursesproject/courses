pub mod parser;

use crate::common::PosInfo;
use pulldown_cmark::{HeadingLevel, LinkType};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Inline {
    /// Plain text
    Text(String),
    Styled(Vec<Inline>, Style),
    /// Inline code
    Code(String),
    /// A code block. May originate from markdown fenced code blocks or notebook code cells.
    CodeBlock {
        /// Code source
        source: String,
        /// Code tags
        tags: Option<Vec<String>>,
        /// Meta
        meta: CodeMeta,
        /// Display the block as a cell or listing (only used for notebooks)
        display_cell: bool,
        global_idx: usize,
        pos: PosInfo,
    },
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
    Math {
        source: String,
        display_block: bool,
        pos: PosInfo,
    },
    Command {
        function: String,
        id: Option<String>,
        parameters: Vec<Parameter>,
        body: Option<Vec<Block>>,
        pos: PosInfo,
        global_idx: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub key: Option<String>,
    pub value: Value,
    pub pos: PosInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Flag(String),
    Content(Vec<Block>),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CodeMeta {
    pub id: String,
    pub editable: bool,
    pub folded: bool,
    pub custom: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Style {
    Emphasis,
    Strong,
    Strikethrough,
    Underline,
}

// pub fn vec_inline_to_string<'a>(vec: &'a [Inline<'a>]) -> Cow<'a, str> {
//     vec.iter().map(|item| item.to_cow_string()).collect()
// }
// impl<'a> Inline<'a> {
//     fn to_cow_string(&self) -> Cow<'a, str> {
//         match self {
//             Inline::Text(s) => s.clone(),
//             Inline::Styled(inner, _) => vec_inline_to_string(inner),
//             Inline::Code(s) => s.clone(),
//             Inline::SoftBreak => String::default(),
//             Inline::HardBreak => String::default(),
//             Inline::Rule => String::default(),
//             Inline::Html(s) => s.to_string(),
//             _ => String::default(),
//         }
//     }
// }
