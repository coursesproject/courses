use crate::ast::AEvent;
use crate::document::EventDocument;
use crate::processors::{
    Error, EventProcessor, EventProcessorConfig, Preprocessor, ProcessorContext,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EscapeProcessorConfig;

#[typetag::serde(name = "escapes")]
impl EventProcessorConfig for EscapeProcessorConfig {
    fn build(&self, ctx: &ProcessorContext) -> anyhow::Result<Box<dyn EventProcessor>> {
        Ok(Box::new(EscapeProcessor))
    }
}

#[derive(Debug)]
pub struct EscapeProcessor;

impl EventProcessor for EscapeProcessor {
    fn name(&self) -> String {
        "Escape processor".to_string()
    }

    fn process(&self, input: EventDocument) -> Result<EventDocument, Error> {
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
        Ok(EventDocument {
            metadata: input.metadata,
            variables: input.variables,
            content: iter.collect(),
        })
    }
}
