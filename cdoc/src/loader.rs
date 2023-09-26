use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::ast::Ast;
use anyhow::{anyhow, Context};
use thiserror::Error;

use crate::document::{split_shortcodes, Document, DocumentMetadata};
use crate::notebook::Notebook;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Std(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// A loader creates an initial [Document] from a raw input string loaded from a content file.
#[typetag::serde(tag = "type")]
pub trait Loader: Debug {
    /// Perform any parsing/conversion necessary.
    fn load(&self, input: &str) -> anyhow::Result<Document<Ast>>;
}

/// Parses a Jupyter Notebook file (.ipynb).
#[derive(Serialize, Deserialize, Debug)]
pub struct NotebookLoader;

#[typetag::serde(name = "notebook_loader")]
impl Loader for NotebookLoader {
    fn load(&self, input: &str) -> anyhow::Result<Document<Ast>> {
        let nb: Notebook = serde_json::from_str(input)?;
        nb.try_into()
    }
}

/// Loads a markdown document. It reads the yml frontmatter and creates the document from the remaining input.
#[derive(Serialize, Deserialize, Debug)]
pub struct MarkdownLoader;

#[typetag::serde(name = "markdown_loader")]
impl Loader for MarkdownLoader {
    fn load(&self, input: &str) -> anyhow::Result<Document<Ast>> {
        // let yml: yaml_front_matter::Document<DocumentMetadata> =
        //     // YamlFrontMatter::parse(input).map_err(|_e| anyhow!("Could not parse front matter"))?;
        //     YamlFrontMatter::parse(input)?;
        let start = input
            .find("---")
            .ok_or_else(|| anyhow!("Missing frontmatter specifier"))?;
        let end = start
            + 3
            + input[start + 3..]
                .find("---")
                .ok_or_else(|| anyhow!("Missing frontmatter specifier"))?;

        let meta: DocumentMetadata =
            serde_yaml::from_str(&input[start + 3..end]).context("Could not parse frontmatter")?;
        let mut counters = HashMap::new();
        let elems = split_shortcodes(&input[end + 3..], 0, 0, &mut counters)?;
        // let elems: Vec<Element> = split_shortcodes_old(&input[end + 3..], &mut counters)?
        //     .into_iter()
        //     .flat_map(|e| match e {
        //         Element::Markdown { content } => split_markdown_old(&content),
        //         _ => vec![e],
        //     })
        //     .collect();
        Ok(Document::new(Ast(elems), meta, counters))
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    //
    // #[test]
    // fn test_deserialization() {
    //     let config = r#"
    //         {
    //             ".md": {"type": "markdown_loader"},
    //             ".ipynb": {"type": "notebook_loader"}
    //         }
    //     "#;
    //
    //     let p: LoaderConfig = serde_json::from_str(config).unwrap();
    // }
}
