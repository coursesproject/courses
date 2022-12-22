use dyn_clone::DynClone;
use std::fmt::{Debug, Display};

use tera::Tera;
use thiserror::Error;

use crate::config::OutputFormat;
use crate::document::{DocPos, Document, EventContent};
use crate::parsers::split::Rule;

mod escapes;
pub mod exercises;
pub mod katex;
pub mod shortcodes;

#[derive(Error, Debug)]
pub enum Error {
    #[error("code split syntax error at {}: {}", .1, .0)]
    CodeParseError(#[source] Box<pest::error::Error<Rule>>, DocPos),
    #[error("could not parse attributes: {}", .0)]
    AttrParseError(#[from] toml::de::Error),
}

#[derive(Clone, Debug)]
pub struct PreprocessorContext {
    pub tera: Tera,
    pub output_format: OutputFormat,
}

pub trait MarkdownPreprocessor: Display {
    fn name(&self) -> String;
    fn process(&self, input: &str, ctx: &tera::Context) -> Result<String, anyhow::Error>;
}

pub trait EventPreprocessor: Display {
    fn name(&self) -> String;
    fn process(&self, input: Document<EventContent>) -> Result<Document<EventContent>, Error>;
}

#[typetag::serde(tag = "type")]
pub trait PreprocessorConfig: Debug + Send + Sync + DynClone {
    fn build(&self, ctx: &PreprocessorContext) -> anyhow::Result<Box<dyn MarkdownPreprocessor>>;
}

#[typetag::serde(tag = "type")]
pub trait EventPreprocessorConfig: Debug + Send + Sync + DynClone {
    fn build(&self, ctx: &PreprocessorContext) -> anyhow::Result<Box<dyn EventPreprocessor>>;
}

dyn_clone::clone_trait_object!(PreprocessorConfig);
dyn_clone::clone_trait_object!(EventPreprocessorConfig);
