pub mod code_split;
mod escapes;
pub mod katex;
pub mod shortcode_extender;

use crate::document::{ConfigureCollector, DocPos, EventDocument};
use crate::parsers::split::Rule;
use crate::Context;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("code split syntax error at {}: {}", .1, .0)]
    CodeParseError(#[source] Box<pest::error::Error<Rule>>, DocPos),
    #[error("could not parse attributes: {}", .0)]
    AttrParseError(#[from] toml::de::Error),
}

#[typetag::serde(tag = "type")]
pub trait Preprocessor: Debug {
    fn name(&self) -> String;
    fn process(&self, input: &str, ctx: &Context) -> Result<String, Box<dyn std::error::Error>>;
}

#[typetag::serde(tag = "type")]
pub trait EventProcessor: Debug {
    fn name(&self) -> String;
    fn process(&self, input: EventDocument) -> Result<EventDocument, Error>;
}
