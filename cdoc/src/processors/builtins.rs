use std::fmt::{Display, Formatter};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use crate::ast::{ACodeBlockKind, AEvent, ATag};
use crate::document::{DocPos, Document, EventContent};
use crate::processors::{Error, EventPreprocessor, EventPreprocessorConfig, PreprocessorContext};
use crate::processors::shortcodes::{ShortCode, ShortCodeRenderer};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuiltinsConfig;

#[typetag::serde(name = "builtins")]
impl EventPreprocessorConfig for BuiltinsConfig {
    fn build(&self, ctx: &PreprocessorContext) -> anyhow::Result<Box<dyn EventPreprocessor>> {
        Ok(Box::new(Builtins {
            renderer: ShortCodeRenderer {
                tera: ctx.tera.clone(),
                file_ext: ctx.output_format.template_extension().to_string(),
            }
        }))
    }
}

#[derive(Debug)]
pub struct Builtins {
    renderer: ShortCodeRenderer,
}

impl EventPreprocessor for Builtins {
    fn name(&self) -> String {
        "builtins".to_string()
    }

    fn process(&self, input: Document<EventContent>, ctx: &tera::Context) -> Result<Document<EventContent>, Error> {
        let mut code_block = false;
        let mut code_attr = String::new();
        let mut source = String::new();

        let mut ref_count = 0;

        let content = input.content.into_iter().flat_map(|(event, pos)| {
            match event {
                AEvent::Start(tag) => {
                    if let ATag::CodeBlock(ACodeBlockKind::Fenced(attr)) = &tag {
                        code_block = true;
                        code_attr = attr.to_string();
                    }
                    vec![]
                }
                AEvent::End(ref tag) => {
                    if let ATag::CodeBlock(ACodeBlockKind::Fenced(_)) = tag {
                        let code = ShortCode::new("cell").with_param("source", source.clone()).with_param("ref", ref_count.to_string());
                        let res = self.renderer.render(&code, ctx).context("could not render template for code cell").unwrap(); //TODO

                        ref_count += 1;

                        code_block = false;
                        source = String::new();


                        vec![
                            Ok((AEvent::Html(res), pos)),
                        ]
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
            }
        }).collect::<Result<Vec<(AEvent, DocPos)>, Error>>()?;

        Ok(Document {
            metadata: input.metadata,
            variables: input.variables,
            content,
        })
    }
}

impl Display for Builtins {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}