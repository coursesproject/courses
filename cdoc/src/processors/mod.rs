use std::fmt::Debug;

use tera::Tera;
use thiserror::Error;

use crate::config::OutputFormat;
use crate::document::{DocPos, EventDocument};
use crate::parsers::split::Rule;

pub mod code_split;
mod escapes;
pub mod katex;
pub mod shortcode_extender;

#[derive(Error, Debug)]
pub enum Error {
    #[error("code split syntax error at {}: {}", .1, .0)]
    CodeParseError(#[source] Box<pest::error::Error<Rule>>, DocPos),
    #[error("could not parse attributes: {}", .0)]
    AttrParseError(#[from] toml::de::Error),
}

pub struct ProcessorContext {
    pub tera: Tera,
    pub output_format: OutputFormat,
}

pub trait Preprocessor {
    fn name(&self) -> String;
    fn process(&self, input: &str, ctx: &tera::Context) -> Result<String, anyhow::Error>;
}

pub trait EventProcessor {
    fn name(&self) -> String;
    fn process(&self, input: EventDocument) -> Result<EventDocument, Error>;
}

#[typetag::serde(tag = "type")]
pub trait PreprocessorConfig: Debug + Send + Sync {
    fn build(&self, ctx: &ProcessorContext) -> anyhow::Result<Box<dyn Preprocessor>>;
}

#[typetag::serde(tag = "type")]
pub trait EventProcessorConfig: Debug + Send + Sync {
    fn build(&self, ctx: &ProcessorContext) -> anyhow::Result<Box<dyn EventProcessor>>;
}
