use crate::ast::AEvent;
use crate::document::EventDocument;
use crate::processors::{Error, EventProcessor};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Serialize, Deserialize, Debug)]
pub struct EscapeProcessor;

#[typetag::serde(name = "escapes")]
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
            content: iter.collect(),
        })
    }
}
