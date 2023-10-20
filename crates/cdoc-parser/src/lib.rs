pub mod code_ast;
mod common;

pub use common::*;
pub mod notebook;
pub mod raw;

pub mod parser;
#[cfg(feature = "scripting")]
pub mod scripting;

fn parse_raw(doc: RawDocument) -> anyhow::Result<Document<Vec<Node>>> {
    let composed = ComposedMarkdown::from(doc.src);
    let code_outputs = composed
        .children
        .iter()
        .filter_map(|c| match &c.elem {
            Special::CodeBlock { inner, .. } => Some((c.label.to_string(), CodeOutput::default())),
            _ => None,
        })
        .collect();

    let nodes: Vec<Node> = Vec::from(composed);

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

pub fn try_doc_from_str(value: &str) -> anyhow::Result<Document<Vec<Node>>> {
    let raw = parse_to_doc(value)?;
    parse_raw(raw)
}

use crate::raw::{parse_to_doc, ComposedMarkdown, RawDocument, Special};
use cdoc_base::document::{CodeOutput, Document, Metadata};
use cdoc_base::node::Node;
#[cfg(test)]
use pest_test_gen::pest_tests;

#[pest_tests(
    crate::raw::RawDocParser,
    crate::raw::Rule,
    "doc",
    dir = "tests/pest/doc",
    strict = false,
    lazy_static = true
)]
#[cfg(test)]
mod raw_doc_tests {}
