use crate::processors::PreprocessorContext;
use crate::processors::{MarkdownPreprocessor, PreprocessorConfig};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use tera::Context;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EscapesConfig;

#[typetag::serde(name = "escapes")]
impl PreprocessorConfig for EscapesConfig {
    fn build(&self, _ctx: &PreprocessorContext) -> anyhow::Result<Box<dyn MarkdownPreprocessor>> {
        Ok(Box::new(Escapes))
    }
}

#[derive(Debug)]
pub struct Escapes;

impl Display for Escapes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl MarkdownPreprocessor for Escapes {
    fn name(&self) -> String {
        "escapes".to_string()
    }

    fn process(&self, input: &str, _ctx: &Context) -> Result<String, Error> {
        Ok(input.replace('\\', r#"\\"#))
    }
}
