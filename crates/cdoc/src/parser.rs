use cdoc_base::node::Element;
use cdoc_parser::ast::Ast;
use cdoc_parser::document::Document;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::preprocessors::{AstPreprocessor, AstPreprocessorConfig, PreprocessorContext};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Parser {
    #[serde(default = "default_preprocessors")]
    pub preprocessors: Vec<Box<dyn AstPreprocessorConfig>>,
    #[serde(default, flatten)]
    pub settings: ParserSettings,
}

fn default_preprocessors() -> Vec<Box<dyn AstPreprocessorConfig>> {
    vec![]
}

/// Additional parser configuration.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ParserSettings {
    /// Include solutions for the [crate::preprocessors::exercises::Exercises] preprocessor.
    #[serde(default)]
    pub solutions: bool,
}

impl Parser {
    pub fn parse(
        &self,
        doc: Document<Vec<Element>>,
        ctx: &PreprocessorContext,
    ) -> Result<Document<Vec<Element>>, anyhow::Error> {
        let doc_ast = self.run_ast_processors(doc.clone(), ctx)?;

        Ok(doc_ast)
    }

    pub fn run_ast_processors(
        &self,
        doc: Document<Vec<Element>>,
        ctx: &PreprocessorContext,
    ) -> Result<Document<Vec<Element>>, anyhow::Error> {
        let mut built = self
            .preprocessors
            .iter()
            .map(|p| p.build(ctx, &self.settings))
            .collect::<anyhow::Result<Vec<Box<dyn AstPreprocessor>>>>()?;

        let doc = built
            .iter_mut()
            .try_fold(doc, |c, ast_processor| ast_processor.process(c))?;

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
    ExtensionError(#[from] crate::preprocessors::Error),

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
