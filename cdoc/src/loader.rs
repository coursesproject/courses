use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use yaml_front_matter::YamlFrontMatter;

use anyhow::{anyhow, Context};

use crate::document::{Document, DocumentMetadata, RawContent};
use crate::notebook::Notebook;

/// A loader creates an initial [Document] from a raw input string loaded from a content file.
#[typetag::serde(tag = "type")]
pub trait Loader: Debug {
    /// Perform any parsing/conversion necessary.
    fn load(&self, input: &str) -> Result<Document<RawContent>, anyhow::Error>;
}

/// Parses a Jupyter Notebook file (.ipynb).
#[derive(Serialize, Deserialize, Debug)]
pub struct NotebookLoader;

#[typetag::serde(name = "notebook_loader")]
impl Loader for NotebookLoader {
    fn load(&self, input: &str) -> Result<Document<RawContent>, anyhow::Error> {
        let nb: Notebook = serde_json::from_str(input)?;
        let meta = nb
            .get_front_matter()
            .context("Failed to read front matter")?;
        Ok(Document::new(nb, meta))
    }
}

/// Loads a markdown document. It reads the yml frontmatter and creates the document from the remaining input.
#[derive(Serialize, Deserialize, Debug)]
pub struct MarkdownLoader;

#[typetag::serde(name = "markdown_loader")]
impl Loader for MarkdownLoader {
    fn load(&self, input: &str) -> Result<Document<RawContent>, anyhow::Error> {
        let yml: yaml_front_matter::Document<DocumentMetadata> =
            YamlFrontMatter::parse(input).map_err(|e| anyhow!("Could not parse front matter"))?;
        Ok(Document::new(yml.content.clone(), yml.metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialization() {
        let config = r#"
            {
                ".md": {"type": "markdown_loader"},
                ".ipynb": {"type": "notebook_loader"}
            }
        "#;

        let p: LoaderConfig = serde_json::from_str(config).unwrap();
    }
}
