use dyn_clone::DynClone;
use std::fmt::{Debug, Display};
use std::path::PathBuf;

use cdoc_base::node::Node;
use cdoc_parser::ast::Ast;
use cdoc_parser::document::Document;
use thiserror::Error;

use crate::config::Format;

use crate::parser::ParserSettings;
use crate::templates::TemplateManager;

pub mod cell_outputs;
pub mod md_labels;
pub mod script;
pub mod solutions;

#[derive(Error, Debug)]
pub enum Error {
    // #[error("code split syntax error at {}", .0)]
    // CodeParseError(#[source] Box<pest::error::Error<Rule>>),
    #[error("could not parse attributes: {}", .0)]
    AttrParseError(#[from] toml::de::Error),

    #[error(transparent)]
    Any(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct PreprocessorContext<'a> {
    pub project_root: PathBuf,
    pub templates: &'a TemplateManager,
    pub output_format: &'a dyn Format,
}

pub trait AstPreprocessor: Display {
    fn name(&self) -> String;
    fn process(&mut self, input: Document<Vec<Node>>) -> Result<Document<Vec<Node>>, Error>;
}

#[typetag::serde]
pub trait AstPreprocessorConfig: Debug + Send + Sync + DynClone {
    fn build(
        &self,
        ctx: &PreprocessorContext,
        settings: &ParserSettings,
    ) -> anyhow::Result<Box<dyn AstPreprocessor>>;
}

dyn_clone::clone_trait_object!(AstPreprocessorConfig);
