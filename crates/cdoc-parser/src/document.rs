use crate::ast::Ast;
use crate::raw::{parse_to_doc, ComposedMarkdown, RawDocument, Special};
use anyhow::Result;
use cdoc_base::node::Element;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Document<T: Serialize> {
    pub meta: Metadata,
    pub content: T,
    pub code_outputs: HashMap<u64, CodeOutput>,
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
    pub code_solutions: Option<bool>,
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
    pub user_defined: LinkedHashMap<String, Value>,
}

const fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LayoutSettings {
    pub hide_sidebar: bool,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct CodeOutput {
    pub values: Vec<OutputValue>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum OutputValue {
    Plain(String),
    Text(String),
    Image(Image),
    Json(Value),
    Html(String),
    Javascript(String),
    Error(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Image {
    Png(String),
    Svg(String),
}

fn parse_raw(doc: RawDocument) -> Result<Document<Vec<Element>>> {
    let composed = ComposedMarkdown::from(doc.src);
    let code_outputs = composed
        .children
        .iter()
        .filter_map(|c| match &c.elem {
            Special::CodeBlock { inner, .. } => Some((inner.hash, CodeOutput::default())),
            _ => None,
        })
        .collect();

    let nodes: Vec<Element> = Vec::from(composed);

    let doc = Document {
        content: nodes,
        meta: doc.meta.map_or(
            Ok::<Metadata, serde_yaml::Error>(Metadata::default()),
            |meta| serde_yaml::from_str(&meta),
        )?,
        code_outputs,
    };

    Ok(doc)
}

impl TryFrom<&str> for Document<Vec<Element>> {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let raw = parse_to_doc(value)?;
        parse_raw(raw)
    }
}

impl<T: Serialize> Document<T> {
    pub fn map<O: Serialize, F: Fn(T) -> O>(self, f: F) -> Document<O> {
        Document {
            content: f(self.content),
            meta: self.meta,
            code_outputs: self.code_outputs,
            // references: self.references,
            // references_by_type: self.references_by_type,
        }
    }

    pub fn try_map<O: Serialize, F: Fn(T) -> Result<O>>(self, f: F) -> Result<Document<O>> {
        Ok(Document {
            content: f(self.content)?,
            meta: self.meta,
            code_outputs: self.code_outputs,
            // references: self.references,
            // references_by_type: self.references_by_type,
        })
    }
}
