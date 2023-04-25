mod collectors;
mod events;
mod iterators;
mod visitor;

pub use collectors::*;
pub use events::*;
pub use visitor::*;

use crate::notebook::CellOutput;
use pulldown_cmark::{HeadingLevel, LinkType, Options, Parser};
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
    Math(String, bool, bool),
    Shortcode(Shortcode),
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
            Inline::Math(s, _, _) => s.to_string(),
            Inline::Shortcode(s) => "shortcode".to_string(),
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
        tags: Option<Vec<String>>,
        outputs: Vec<CellOutput>,
    },
    List(Option<u64>, Vec<Block>),
    ListItem(Vec<Block>),
}

#[derive(Debug, Clone)]
pub enum Shortcode {
    Inline(ShortcodeBase),
    Block(ShortcodeBase, Vec<Block>),
}

pub fn str_to_blocks(input: &str) -> Vec<Block> {
    let ast: Ast = Parser::new_ext(input, Options::all()).collect();
    ast.0
}

pub fn math_block_md(src: &str, display_block: bool, trailing_space: bool) -> String {
    let delim = if display_block { "$$" } else { "$" };
    let trail = if trailing_space { " " } else { "" };
    format!("{}{}{}{}", delim, src, delim, trail)
}

#[derive(Debug, Clone)]
pub struct ShortcodeBase {
    pub(crate) name: String,
    pub(crate) id: Option<String>,
    pub(crate) num: usize,
    pub(crate) parameters: HashMap<String, Vec<Block>>,
}

pub enum ShortcodeIdx {
    Inline(usize, usize),
    Block {
        def: (usize, usize),
        end: (usize, usize),
    },
}

fn extract_block(start: usize, input: &str) -> Option<ShortcodeIdx> {
    let end = start + input[start..].find("%}")?;

    let mut level = 1;
    let mut cur_start = end;

    let mut end_block = end;

    while level > 0 {
        let new_start = input[(cur_start + 2)..]
            .find("{%")
            .map(|s| s + cur_start + 2);
        end_block = cur_start + 2 + input[cur_start + 2..].find("{% end")?;
        match new_start {
            Some(s) => {
                if s < end_block {
                    level += 1;
                    cur_start = s + 2;
                } else {
                    level -= 1;
                    cur_start = end_block + 7;
                }
            }
            None => {
                level -= 1;
                cur_start = end_block + 7;
            }
        }
    }

    let end_of_block = end_block + 6 + input[end_block + 6..].find("%}")?;

    Some(ShortcodeIdx::Block {
        def: (start, end),
        end: (end_block, end_of_block),
    })
}

fn extract_inline(start: usize, input: &str) -> Option<ShortcodeIdx> {
    let mut level = 1;
    let mut end = start;
    let mut cur_start = start;
    while level > 0 {
        let new_start = input[(cur_start + 2)..]
            .find("{{")
            .map(|s| s + cur_start + 2);
        end = cur_start + 2 + input[(cur_start + 2)..].find("}}")?;
        match new_start {
            Some(s) => {
                if s < end {
                    level += 1;
                    cur_start = s + 2;
                } else {
                    level -= 1;
                    cur_start = end + 2;
                }
            }
            None => {
                level -= 1;
                cur_start = end + 2;
            }
        }
    }
    Some(ShortcodeIdx::Inline(start, end))
}

pub fn find_all_blocks(input: &str) -> Vec<(usize, usize)> {
    let mut rest = input;
    let mut offset = 0;

    let mut res = Vec::new();
    loop {
        let next = find_next_block(rest);
        match next {
            None => return res,
            Some((start, end)) => {
                res.push((offset + start, offset + end));
                rest = &rest[(end)..];
                offset += end;
            }
        }
    }
}

fn find_next_block(input: &str) -> Option<(usize, usize)> {
    let start = input.find('`')?;
    let end_delim = if input[(start + 1)..].len() > 2 && &input[(start + 1)..(start + 3)] == "``" {
        "```"
    } else {
        "`"
    };

    let end = start + 1 + input[(start + 1)..].find(end_delim)? + end_delim.len();
    Some((start, end))
}

pub fn find_shortcode(input: &str) -> Option<ShortcodeIdx> {
    let start_inline = input.find("{{");
    let start_block = input.find("{%");

    match start_inline {
        None => start_block.and_then(|start| extract_block(start, input)),
        Some(inline_start_idx) => match start_block {
            None => extract_inline(inline_start_idx, input),
            Some(block_start_idx) => {
                if inline_start_idx < block_start_idx {
                    extract_inline(inline_start_idx, input)
                } else {
                    extract_block(block_start_idx, input)
                }
            }
        },
    }
}
