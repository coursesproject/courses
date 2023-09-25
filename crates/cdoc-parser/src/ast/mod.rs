pub mod parser;
pub mod visitor;

use crate::code_ast::types::CodeContent;
use crate::common::Span;
use cowstr::CowStr;
use pulldown_cmark::LinkType;
use serde::{Deserialize, Serialize};

use linked_hash_map::LinkedHashMap;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Ast {
    pub blocks: Vec<Block>,
    pub source: CowStr,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Command {
    pub function: CowStr,
    pub label: Option<CowStr>,
    pub parameters: Vec<Parameter>,
    pub body: Option<Vec<Block>>,
    pub span: Span,
    pub global_idx: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CodeBlock {
    /// Label
    pub label: Option<CowStr>,
    /// Code source
    pub source: CodeContent,
    /// Code tags
    pub attributes: Vec<CowStr>,
    /// Display the block as a cell or listing (only used for notebooks)
    pub display_cell: bool,
    pub global_idx: usize,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Math {
    pub source: CowStr,
    pub label: Option<CowStr>,
    pub display_block: bool,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Inline {
    /// Plain text
    Text(CowStr),
    Styled(Vec<Inline>, Style),
    /// Inline code
    Code(CowStr),
    /// A code block. May originate from markdown fenced code blocks or notebook code cells.
    CodeBlock(CodeBlock),
    SoftBreak,
    HardBreak,
    /// Horizontal rule
    Rule,
    /// An inline image (usually originates from a markdown image spec)
    Image(LinkType, CowStr, CowStr, Vec<Inline>),
    /// An inline link (usually originates from a markdown link spec)
    Link(LinkType, CowStr, CowStr, Vec<Inline>),
    /// Unescaped html.
    Html(CowStr),
    /// Math element (may be inline or display)
    /// The trailing space element is necessary due to the way parsing currently works with
    /// pulldown_cmark.
    Math(Math),
    Command(Command),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub key: Option<CowStr>,
    pub value: Value,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub obj_type: String,
    pub attr: LinkedHashMap<CowStr, CowStr>,
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
    Flag(CowStr),
    Content(Vec<Block>),
    String(CowStr),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Block {
    Heading {
        lvl: u8,
        id: Option<CowStr>,
        classes: Vec<CowStr>,
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
