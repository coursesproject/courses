use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::ast::AEvent;
use crate::document::{Document, EventContent};
use crate::processors::{Error, EventPreprocessor, EventPreprocessorConfig, PreprocessorContext};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EscapesConfig;

#[typetag::serde(name = "escapes")]
impl EventPreprocessorConfig for EscapesConfig {
    fn build(&self, _ctx: &PreprocessorContext) -> anyhow::Result<Box<dyn EventPreprocessor>> {
        Ok(Box::new(Escapes))
    }
}

#[derive(Debug)]
pub struct Escapes;

impl EventPreprocessor for Escapes {
    fn name(&self) -> String {
        "Escape processor".to_string()
    }

    fn process(&self, input: Document<EventContent>) -> Result<Document<EventContent>, Error> {
        let iter = input.content.into_iter().map(|(e, pos)| {
            (
                if let AEvent::Text(txt) = e {
                    if &txt == "\\" {
                        AEvent::Text("\\\\".to_string())
                    } else {
                        AEvent::Text(txt)
                    }
                } else {
                    e
                },
                pos,
            )
        });
        Ok(Document {
            metadata: input.metadata,
            variables: input.variables,
            content: iter.collect(),
        })
    }
}

impl Display for Escapes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
