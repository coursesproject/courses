use dyn_clone::DynClone;
use std::fmt::{Debug, Display};

use crate::ast::Ast;

use thiserror::Error;

use crate::config::Format;
use crate::document::Document;
use crate::parsers::split::Rule;
use crate::templates::TemplateManager;

pub mod exercises;

#[derive(Error, Debug)]
pub enum Error {
    #[error("code split syntax error at {}", .0)]
    CodeParseError(#[source] Box<pest::error::Error<Rule>>),
    #[error("could not parse attributes: {}", .0)]
    AttrParseError(#[from] toml::de::Error),

    #[error(transparent)]
    Any(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct PreprocessorContext<'a> {
    pub templates: &'a TemplateManager,
    pub output_format: &'a dyn Format,
}

pub trait AstPreprocessor: Display {
    fn name(&self) -> String;
    fn process(&mut self, input: Document<Ast>) -> Result<Document<Ast>, Error>;
}

#[typetag::serde(tag = "name")]
pub trait AstPreprocessorConfig: Debug + Send + Sync + DynClone {
    fn build(&self, ctx: &PreprocessorContext) -> anyhow::Result<Box<dyn AstPreprocessor>>;
}

dyn_clone::clone_trait_object!(AstPreprocessorConfig);
