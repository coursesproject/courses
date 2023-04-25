use crate::ast::Ast;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::document::{Document, PreprocessError};
use crate::processors::shortcodes::ShortCodeProcessError;
use crate::processors::{
    AstPreprocessor, AstPreprocessorConfig, MarkdownPreprocessor, PreprocessorConfig,
    PreprocessorContext,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parser {
    #[serde(default)]
    pub md_processors: Vec<Box<dyn PreprocessorConfig>>,
    #[serde(default)]
    pub ast_processors: Vec<Box<dyn AstPreprocessorConfig>>,
    pub settings: ParserSettings,
}

/// Additional parser configuration.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParserSettings {
    /// Include solutions for the [Exercises] preprocessor.
    #[serde(default)]
    pub solutions: bool,
    /// Include notebook outputs (from cells) in the loaded output.
    #[serde(default)]
    pub notebook_outputs: bool,
}

impl Parser {
    pub fn parse(
        &self,
        doc: &Document<Ast>,
        template_context: &tera::Context,
        ctx: &PreprocessorContext,
    ) -> Result<Document<Ast>, anyhow::Error> {
        let doc_ast = self.run_ast_processors(doc.clone(), ctx)?;

        Ok(doc_ast)
    }

    pub fn run_ast_processors(
        &self,
        doc: Document<Ast>,
        ctx: &PreprocessorContext,
    ) -> Result<Document<Ast>, anyhow::Error> {
        let mut built = self
            .ast_processors
            .iter()
            .map(|p| p.build(ctx))
            .collect::<anyhow::Result<Vec<Box<dyn AstPreprocessor>>>>()?;

        let doc = built.iter_mut().fold(Ok(doc), |c, ast_processor| {
            c.and_then(|c| ast_processor.process(c))
        })?;

        Ok(doc)
    }
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

    #[cfg(feature = "katex")]
    #[error(transparent)]
    KaTeX(#[from] katex::Error),

    #[error(transparent)]
    Std(#[from] Box<dyn std::error::Error>),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn test_deserialization() {
    //     let config = r#"
    //         {
    //             "preprocessors": [
    //                 {
    //                     "type": "shortcodes",
    //                     "template": "tp/**",
    //                     "file_ext": ".html"
    //                 }
    //             ],
    //             "event_processors": []
    //         }
    //     "#;
    //
    //     let p: Parser = serde_json::from_str(config).unwrap();
    // }
}
