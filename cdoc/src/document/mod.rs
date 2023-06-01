mod visitors;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::Range;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::ast::{
    find_shortcode, str_to_blocks, Ast, AstVisitor, Block, Inline, ShortcodeBase, ShortcodeIdx,
};
use crate::document::visitors::{MathInserter, ShortcodeInserter};
use crate::notebook::{Cell, Notebook};
use crate::parsers::shortcodes::{Argument, ArgumentValue, ShortCodeDef};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default = "default_true")]
    pub exercises: bool,
    #[serde(default)]
    pub code_solutions: bool,
    #[serde(default = "default_true")]
    pub cell_outputs: bool,
    #[serde(default)]
    pub interactive: bool,
    #[serde(default)]
    pub editable: bool,
    #[serde(default)]
    pub layout: LayoutSettings,

    #[serde(default)]
    pub exclude_outputs: Option<Vec<String>>,
}

const fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LayoutSettings {
    pub hide_sidebar: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Document<C> {
    pub content: C,
    pub metadata: DocumentMetadata,
    pub variables: DocumentVariables,
    pub ids: HashMap<String, (usize, Vec<ShortCodeDef>)>,
    pub id_map: HashMap<String, (usize, ShortCodeDef)>,
}

pub fn split_markdown(src: &str) -> Result<Vec<Block>> {
    let mut rest = src;
    let mut is_eq = false;

    let mut math_blocks = Vec::new();
    let mut res = String::new();
    let mut eq_idx = 0;
    while let Some(idx) = rest.find('$') {
        let is_block = rest.len() > 2 && &rest[idx + 1..idx + 2] == "$";
        let trailing_space = rest.len() > 2 && &rest[idx + 1..idx + 2] == " ";

        if is_eq {
            res.push_str(&format!("__{eq_idx}__"));
            math_blocks.push(Inline::Math {
                source: rest[..idx].to_string(),
                display_block: is_block,
                trailing_space,
            });
            eq_idx += 1;
        } else {
            res.push_str(&rest[..idx]);
        }

        is_eq = !is_eq;
        let offset = if is_block { 2 } else { 1 };
        rest = &rest[idx + offset..];
    }

    if !rest.is_empty() {
        res.push_str(rest)
    }

    let mut md_blocks = str_to_blocks(&res)?;

    MathInserter::new(math_blocks).walk_ast(&mut md_blocks)?;

    Ok(md_blocks.0)
}

pub fn split_shortcodes(
    src: &str,
    counters: &mut HashMap<String, (usize, Vec<ShortCodeDef>)>,
) -> Result<Vec<Block>> {
    let mut rest = src;
    let mut md_str = String::new();
    let mut shortcodes = Vec::new();
    let mut shortcode_idx = 0;
    while let Some(info) = find_shortcode(rest) {
        match info {
            ShortcodeIdx::Inline(start, end) => {
                md_str.push_str(&rest[..start]);

                let c = rest[start + 2..end].trim();
                shortcodes.push((c, ""));

                rest = &rest[end + 2..];
            }
            ShortcodeIdx::Block { def, end } => {
                md_str.push_str(&rest[..def.0]);

                let c = rest[def.0 + 2..def.1].trim();
                let body = &rest[def.1 + 2..end.0];
                shortcodes.push((c, body));

                rest = &rest[end.1 + 2..];
            }
        }
        md_str.push_str(&format!("_{shortcode_idx}_"));
        shortcode_idx += 1;
    }

    if !rest.is_empty() {
        md_str.push_str(rest);
    }

    let mut md_blocks = Ast(split_markdown(&md_str)?);

    ShortcodeInserter::new(shortcodes, counters).walk_ast(&mut md_blocks)?;

    Ok(md_blocks.0)
}

impl ShortCodeDef {
    fn into_base(
        self,
        counters: &mut HashMap<String, (usize, Vec<ShortCodeDef>)>,
    ) -> Result<ShortcodeBase> {
        let parameters: Result<Vec<Argument<Vec<Block>>>> = self
            .parameters
            .into_iter()
            .map(|param| {
                param.try_map(|v| {
                    Ok(match v {
                        ArgumentValue::Literal(s) => {
                            ArgumentValue::Literal(vec![Block::Plain(vec![Inline::Text(s)])])
                        }
                        ArgumentValue::Markdown(s) => {
                            let blocks = split_shortcodes(&s, counters)?;
                            let blocks = blocks
                                .into_iter()
                                .map(|b| {
                                    if let Block::Paragraph(i) = b {
                                        Block::Plain(i)
                                    } else {
                                        b
                                    }
                                })
                                .collect();
                            ArgumentValue::Markdown(blocks)
                        }
                    })
                })
            })
            .collect();

        Ok(ShortcodeBase {
            name: self.name.clone(),
            id: self.id,
            num: counters.get(&self.name).unwrap().0,
            parameters: parameters?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DocPos {
    cell_number: Option<usize>,
    #[allow(unused)]
    global_offset: usize,
    line: usize,
    #[allow(unused)]
    local_position: Range<usize>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct DocumentVariables {
    pub first_heading: Option<String>,
}

impl Display for DocPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.cell_number {
            None => write!(f, "line: {}", self.line),
            Some(n) => write!(f, "cell: {}, local position: {}", n, self.line),
        }
    }
}

impl DocPos {
    pub fn new(
        cell_number: Option<usize>,
        global_offset: usize,
        line: usize,
        local_position: Range<usize>,
    ) -> Self {
        DocPos {
            cell_number,
            global_offset,
            line,
            local_position,
        }
    }
}

fn id_map_from_ids(
    ids: &HashMap<String, (usize, Vec<ShortCodeDef>)>,
) -> HashMap<String, (usize, ShortCodeDef)> {
    let mut out = HashMap::new();

    for (_, s) in ids.values() {
        let mut tp_num: usize = 1;
        for s in s {
            if let Some(id) = s.id.as_ref() {
                out.insert(id.clone(), (tp_num, s.clone()));
            }
            tp_num += 1;
        }
    }
    out
}

impl<C> Document<C> {
    pub fn new(
        content: C,
        metadata: DocumentMetadata,
        ids: HashMap<String, (usize, Vec<ShortCodeDef>)>,
    ) -> Self {
        let id_map = id_map_from_ids(&ids);
        Document {
            metadata,
            variables: DocumentVariables::default(),
            content,
            ids,
            id_map,
        }
    }

    pub fn map<O, F: Fn(C) -> O>(self, f: F) -> Document<O> {
        Document {
            content: f(self.content),
            metadata: self.metadata,
            variables: self.variables,
            ids: self.ids,
            id_map: self.id_map,
        }
    }
}

impl TryFrom<Notebook> for Document<Ast> {
    type Error = anyhow::Error;

    fn try_from(value: Notebook) -> std::result::Result<Self, Self::Error> {
        let meta = value
            .get_front_matter()
            .context("Failed to read front matter")?;
        let mut counters = HashMap::new();
        let content: Vec<Block> = value
            .cells
            .into_iter()
            .map(|cell| match cell {
                Cell::Markdown { common } => split_shortcodes(&common.source, &mut counters),
                Cell::Code {
                    common, outputs, ..
                } => Ok(vec![Block::CodeBlock {
                    source: common.source,
                    tags: common.metadata.tags,
                    attr: Default::default(),
                    reference: None,
                    outputs,
                }]),
                Cell::Raw { .. } => Ok(vec![]),
            })
            .collect::<Result<Vec<Vec<Block>>>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok(Document::new(Ast(content), meta, counters))
    }
}
