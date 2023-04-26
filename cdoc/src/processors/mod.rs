use dyn_clone::DynClone;
use std::fmt::{Debug, Display};

use crate::ast::Ast;
use tera::Tera;
use thiserror::Error;

use crate::config::OutputFormat;
use crate::document::Document;
use crate::parsers::split::Rule;

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

#[derive(Clone, Debug)]
pub struct PreprocessorContext {
    pub tera: Tera,
    pub output_format: OutputFormat,
}

pub trait AstPreprocessor: Display {
    fn name(&self) -> String;
    fn process(&mut self, input: Document<Ast>) -> Result<Document<Ast>, Error>;
}

#[typetag::serde(tag = "type")]
pub trait AstPreprocessorConfig: Debug + Send + Sync + DynClone {
    fn build(&self, ctx: &PreprocessorContext) -> anyhow::Result<Box<dyn AstPreprocessor>>;
}

dyn_clone::clone_trait_object!(AstPreprocessorConfig);
