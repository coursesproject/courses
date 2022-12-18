use serde::{Deserialize, Serialize};

use crate::ast::{ACodeBlockKind, AEvent, ATag};
use crate::document::{DocPos, EventDocument};
use crate::parsers::split::{human_errors, parse_code_string};
use crate::processors::Error::CodeParseError;
use crate::processors::{Error, EventProcessor, EventProcessorConfig, ProcessorContext};

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeSplitConfig;

#[typetag::serde(name = "code_split")]
impl EventProcessorConfig for CodeSplitConfig {
    fn build(&self, ctx: &ProcessorContext) -> anyhow::Result<Box<dyn EventProcessor>> {
        Ok(Box::new(CodeSplit))
    }
}

#[derive(Debug)]
pub struct CodeSplit;

impl EventProcessor for CodeSplit {
    fn name(&self) -> String {
        "Code split".to_string()
    }

    fn process(&self, input: EventDocument) -> Result<EventDocument, Error> {
        let mut code_block = false;
        let mut source = "".to_string();
        let mut code_attr = String::new();

        let content = input
            .content
            .into_iter()
            .flat_map(|(event, pos)| match &event {
                AEvent::Start(tag) => {
                    if let ATag::CodeBlock(ACodeBlockKind::Fenced(attr)) = &tag {
                        code_block = true;
                        code_attr = attr.to_string();
                    }
                    vec![Ok((AEvent::Start(tag.clone()), pos))]
                }
                AEvent::End(tag) => {
                    if let ATag::CodeBlock(ACodeBlockKind::Fenced(_)) = tag {
                        // TODO: Here
                        let res = parse_code_string(source.clone().as_ref());
                        code_block = false;
                        source = String::new();
                        match res {
                            Ok(doc) => {
                                let (placeholder, _solution) = doc.split();
                                vec![
                                    Ok((AEvent::Text(placeholder.trim().to_string()), pos.clone())),
                                    Ok((AEvent::End(tag.clone()), pos)),
                                ]
                            }
                            Err(e) => vec![Err(CodeParseError(human_errors(*e), pos))],
                        }
                    } else {
                        vec![Ok((event, pos))]
                    }
                }
                AEvent::Text(txt) => {
                    if code_block {
                        source.push_str(txt.as_ref());
                        vec![]
                    } else {
                        vec![Ok((AEvent::Text(txt.clone()), pos))]
                    }
                }
                _ => vec![Ok((event, pos))],
            })
            .collect::<Result<Vec<(AEvent, DocPos)>, Error>>()?;

        Ok(EventDocument {
            metadata: input.metadata,
            variables: input.variables,
            content,
        })
    }
}
