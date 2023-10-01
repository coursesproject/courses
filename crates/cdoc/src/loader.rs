use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use anyhow::{anyhow, Context};
use cdoc_base::node::Element;
use cdoc_parser::ast::Ast;
use cdoc_parser::document::{Document, Metadata};
use cdoc_parser::notebook::{notebook_to_doc, Notebook};
use thiserror::Error;

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
    fn load(
        &self,
        input: &str,
        accept_draft: bool,
    ) -> anyhow::Result<Option<Document<Vec<Element>>>>;
}

/// Parses a Jupyter Notebook file (.ipynb).
#[derive(Serialize, Deserialize, Debug)]
pub struct NotebookLoader;

#[typetag::serde(name = "notebook_loader")]
impl Loader for NotebookLoader {
    fn load(
        &self,
        input: &str,
        accept_draft: bool,
    ) -> anyhow::Result<Option<Document<Vec<Element>>>> {
        let nb: Notebook =
            serde_json::from_str(input).context(anyhow!("deserializing notebook"))?;
        notebook_to_doc(nb, accept_draft)
    }
}

/// Loads a markdown document. It reads the yml frontmatter and creates the document from the remaining input.
#[derive(Serialize, Deserialize, Debug)]
pub struct MarkdownLoader;

#[typetag::serde(name = "markdown_loader")]
impl Loader for MarkdownLoader {
    fn load(
        &self,
        input: &str,
        accept_draft: bool,
    ) -> anyhow::Result<Option<Document<Vec<Element>>>> {
        if accept_draft {
            Some(Document::try_from(input)).transpose()
        } else {
            let doc: yaml_front_matter::Document<Metadata> =
                yaml_front_matter::YamlFrontMatter::parse(input).unwrap();
            if !doc.metadata.draft {
                Some(Document::try_from(input)).transpose()
            } else {
                Ok(None)
            }
        }
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
