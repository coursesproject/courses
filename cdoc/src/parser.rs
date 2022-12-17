use crate::document::{
    DocumentMetadata, EventDocument, IteratorConfig, PreprocessError, RawDocument,
};
use crate::loader::Loader;
use crate::notebook::Notebook;
use crate::processors::shortcode_extender::ShortCodeProcessError;
use crate::processors::{EventProcessor, Preprocessor};
use crate::Context;
use anyhow::anyhow;
use pulldown_cmark::{Event, Tag};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct Parser {
    pub preprocessors: Vec<Box<dyn Preprocessor>>,
    pub event_processors: Vec<Box<dyn EventProcessor>>,
}

impl Parser {
    pub fn parse(&self, doc: &RawDocument, ctx: &Context) -> Result<EventDocument, ParserError> {
        // let ext = path
        //     .as_ref()
        //     .extension()
        //     .ok_or_else(|| anyhow!("File without extension not supported"))?;
        // let ld = self
        //     .parser_config
        //     .get_parser(ext.to_str().unwrap())
        //     .ok_or_else(|| anyhow!("File type not supported"))?;

        let doc = self.run_preprocessors(&doc, ctx)?;
        self.run_event_processors(&doc)
    }

    pub fn run_preprocessors(
        &self,
        doc: &RawDocument,
        ctx: &Context,
    ) -> Result<RawDocument, ParserError> {
        let content = self
            .preprocessors
            .iter()
            .fold(Ok(doc.clone()), |c, preprocessor| {
                c.and_then(|c| c.preprocess(preprocessor.as_ref(), ctx))
            })?;

        Ok(content)
    }

    pub fn run_event_processors(&self, doc: &RawDocument) -> Result<EventDocument, ParserError> {
        let v = doc.to_events(IteratorConfig {
            include_output: doc.metadata.notebook_output,
            include_solutions: false,
        });
        let events = self
            .event_processors
            .iter()
            .fold(Ok(v), |c, event_processor| {
                c.and_then(|c| event_processor.process(c))
            })?;

        Ok(events)
    }
}

#[derive(Debug, Clone, Default)]
pub struct DocumentParsed {
    pub(crate) title: String,
    pub(crate) frontmatter: DocumentMetadata,
    pub(crate) html: String,
    pub(crate) notebook: Notebook,
    pub(crate) md: String,
}

#[allow(unused)]
struct HeadingNode {
    id: String,
    children: Vec<HeadingNode>,
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("IO Error: ")]
    IoError(#[from] std::io::Error),

    #[error("Error in template")]
    TemplateError(#[from] tera::Error),

    #[error("JSON Error: ")]
    JSONError(#[from] serde_json::error::Error),

    #[error("Error parsing frontmatter: ")]
    FrontMatter(#[from] serde_yaml::Error),

    #[error(transparent)]
    Preprocess(#[from] PreprocessError),

    #[error(transparent)]
    ExtensionError(#[from] crate::processors::Error),

    #[error(transparent)]
    ShortCode(#[from] ShortCodeProcessError),

    #[error(transparent)]
    KaTeX(#[from] katex::Error),

    #[error(transparent)]
    Std(#[from] Box<dyn std::error::Error>),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialization() {
        let config = r#"
            {
                "preprocessors": [
                    {
                        "type": "shortcodes",
                        "template": "tp/**",
                        "file_ext": ".html"
                    }
                ],
                "event_processors": []
            }
        "#;

        let p: Parser = serde_json::from_str(config).unwrap();
    }
}
