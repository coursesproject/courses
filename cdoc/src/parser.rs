use std::rc::Rc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::document::{
    DocumentMetadata, EventDocument, IteratorConfig, PreprocessError, RawDocument,
};
use crate::loader::Loader;
use crate::notebook::Notebook;
use crate::processors::shortcode_extender::ShortCodeProcessError;
use crate::processors::{
    EventProcessor, EventProcessorConfig, Preprocessor, PreprocessorConfig, ProcessorContext,
};
use crate::Meta;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParserSettings {
    #[serde(default)]
    pub(crate) solutions: bool,
    #[serde(default)]
    pub(crate) notebook_outputs: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parser {
    pub preprocessors: Vec<Rc<dyn PreprocessorConfig>>,
    pub event_processors: Vec<Rc<dyn EventProcessorConfig>>,
    pub settings: ParserSettings,
}

impl Parser {
    pub fn parse(
        &self,
        doc: &RawDocument,
        template_context: &tera::Context,
        ctx: &ProcessorContext,
    ) -> Result<EventDocument, anyhow::Error> {
        let doc = self.run_preprocessors(&doc, template_context, ctx)?;
        self.run_event_processors(&doc, ctx)
    }

    pub fn run_preprocessors(
        &self,
        doc: &RawDocument,
        template_context: &tera::Context,
        ctx: &ProcessorContext,
    ) -> Result<RawDocument, anyhow::Error> {
        let built = self
            .preprocessors
            .iter()
            .map(|p| p.build(ctx))
            .collect::<anyhow::Result<Vec<Box<dyn Preprocessor>>>>()?;

        let content = built.iter().fold(Ok(doc.clone()), |c, preprocessor| {
            c.and_then(|c| c.preprocess(preprocessor.as_ref(), template_context))
        })?;

        Ok(content)
    }

    pub fn run_event_processors(
        &self,
        doc: &RawDocument,
        ctx: &ProcessorContext,
    ) -> Result<EventDocument, anyhow::Error> {
        let v = doc.to_events(IteratorConfig {
            include_output: doc
                .metadata
                .notebook_output
                .unwrap_or(self.settings.notebook_outputs),
            include_solutions: doc
                .metadata
                .code_solutions
                .unwrap_or(self.settings.solutions),
        });

        let built = self
            .event_processors
            .iter()
            .map(|p| p.build(ctx))
            .collect::<anyhow::Result<Vec<Box<dyn EventProcessor>>>>()?;

        let events = built.iter().fold(Ok(v), |c, event_processor| {
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
