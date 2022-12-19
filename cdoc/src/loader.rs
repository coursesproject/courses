use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use yaml_front_matter::YamlFrontMatter;

use crate::document::{DocumentMetadata, RawDocument};
use crate::notebook::Notebook;

#[typetag::serde(tag = "type")]
pub trait Loader: Debug {
    fn load(&self, input: &str) -> Result<RawDocument, anyhow::Error>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotebookLoader;

#[typetag::serde(name = "notebook_loader")]
impl Loader for NotebookLoader {
    fn load(&self, input: &str) -> Result<RawDocument, anyhow::Error> {
        let nb: Notebook = serde_json::from_str(input)?;
        let meta = nb.get_front_matter()?;
        Ok(RawDocument::new(nb, meta))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MarkdownLoader;

#[typetag::serde(name = "markdown_loader")]
impl Loader for MarkdownLoader {
    fn load(&self, input: &str) -> Result<RawDocument, anyhow::Error> {
        let yml: yaml_front_matter::Document<DocumentMetadata> =
            YamlFrontMatter::parse(input).unwrap();
        Ok(RawDocument::new(yml.content.clone(), yml.metadata))
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoaderConfig {
    #[serde(flatten)]
    mapping: HashMap<String, Box<dyn Loader>>,
}

impl LoaderConfig {
    pub fn add_mapping(&mut self, extension: &str, parser: Box<dyn Loader>) {
        self.mapping.insert(extension.to_string(), parser);
    }

    pub fn get_parser(&self, extension: &str) -> Option<&dyn Loader> {
        self.mapping.get(extension).map(|b| b.deref())
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
