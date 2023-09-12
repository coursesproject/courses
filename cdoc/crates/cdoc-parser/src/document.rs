use crate::ast::Block;
use crate::raw::{parse_to_doc, ComposedMarkdown, RawDocument};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Document {
    pub meta: Metadata,
    pub content: Vec<Block>,
    pub code_outputs: Vec<CodeOutput>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub title: String,
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

    #[serde(flatten)]
    pub user_defined: HashMap<String, Value>,
}

const fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LayoutSettings {
    pub hide_sidebar: bool,
}

#[derive(Debug, PartialEq)]
pub struct CodeOutput {
    pub(crate) values: Vec<Outval>,
}

#[derive(Debug, PartialEq)]
pub enum Outval {
    Text(String),
    Image(Image),
    Json(Value),
    Html(String),
    Javascript(String),
    Error(String),
}

#[derive(Debug, PartialEq)]
pub enum Image {
    Png(String),
    Svg(String),
}

fn parse_raw(doc: RawDocument) -> Result<Document> {
    Ok(Document {
        content: ComposedMarkdown::from(doc.src).into(),
        meta: doc.meta.map_or(
            Ok::<Metadata, serde_yaml::Error>(Metadata::default()),
            |meta| serde_yaml::from_str(&meta),
        )?,
        code_outputs: Vec::new(),
    })
}

impl TryFrom<&str> for Document {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let raw = parse_to_doc(value)?;
        parse_raw(raw)
    }
}
