use crate::parser::ParserSettings;
use crate::preprocessors::{AstPreprocessorConfig, Error, PreprocessorContext, Processor};
use cdoc_base::node::visitor::NodeVisitor;
use cdoc_base::node::{Attribute, Compound, Node};

use cdoc_base::document::{CodeOutput, Document, Image, OutputValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CellOutputConfig;

#[typetag::serde(name = "cells")]
impl AstPreprocessorConfig for CellOutputConfig {
    fn build(
        &self,
        ctx: &PreprocessorContext,
        settings: &ParserSettings,
    ) -> anyhow::Result<Box<dyn Processor>> {
        Ok(Box::new(CellProcessor))
    }
}

#[derive(Debug, Default)]
pub struct CellProcessor;

pub struct CellVisitor<'a> {
    outputs: &'a HashMap<String, CodeOutput>,
}

impl NodeVisitor for CellVisitor<'_> {
    fn visit_compound(&mut self, node: &mut Compound) -> anyhow::Result<()> {
        if node.type_id == "code_block" {
            if let Some(outputs) = self.outputs.get(node.id.as_ref().unwrap()) {
                let mut output_nodes = vec![];
                for output in &outputs.values {
                    match output {
                        OutputValue::Text(s) => {
                            let node = Compound::new_with_attributes(
                                "output_text",
                                None,
                                [(Some("value".to_string()), Attribute::String(s.into()))],
                            );

                            output_nodes.push(Node::Compound(node));
                        }
                        OutputValue::Image(img) => {
                            let mut attributes = node.attributes.clone();

                            match img {
                                Image::Png(png) => attributes.push((
                                    Some("base64".to_string()),
                                    Attribute::String(png.into()),
                                )),
                                Image::Svg(svg) => attributes
                                    .push((Some("svg".to_string()), Attribute::String(svg.into()))),
                            };

                            let node = Compound::new_with_attributes("figure", None, attributes);

                            output_nodes.push(Node::Compound(node));
                        }
                        OutputValue::Json(_) => {}
                        OutputValue::Html(_) => {}
                        OutputValue::Javascript(_) => {}
                        OutputValue::Error(_) => {}
                        OutputValue::Plain(_) => {}
                    }
                }

                node.children.extend(output_nodes);
            }
        }

        self.walk_compound(node)
    }
}

impl Processor for CellProcessor {
    fn name(&self) -> String {
        "cells".to_string()
    }

    fn process(&mut self, mut input: Document<Vec<Node>>) -> Result<Document<Vec<Node>>, Error> {
        if input.meta.cell_outputs {
            let mut visitor = CellVisitor {
                outputs: &input.code_outputs,
            };
            visitor.walk_elements(input.content.as_mut_slice())?;
        }

        Ok(input)
    }
}

impl Display for CellProcessor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
