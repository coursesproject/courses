use crate::parser::ParserSettings;
use crate::processors::{AstPreprocessor, AstPreprocessorConfig, Error, PreprocessorContext};
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Ast, Block, Command, Inline, Parameter, Value};
use cdoc_parser::document::{CodeOutput, Document, Image, Outval};
use cdoc_parser::PosInfo;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CellOutputConfig;

#[typetag::serde(name = "cells")]
impl AstPreprocessorConfig for CellOutputConfig {
    fn build(
        &self,
        _ctx: &PreprocessorContext,
        _settings: &ParserSettings,
    ) -> anyhow::Result<Box<dyn AstPreprocessor>> {
        Ok(Box::new(CellProcessor))
    }
}

#[derive(Debug, Default)]
pub struct CellProcessor;

pub struct CellVisitor<'a> {
    outputs: &'a [CodeOutput],
}

impl AstVisitor for CellVisitor<'_> {
    fn visit_vec_inline(&mut self, inlines: &mut Vec<Inline>) -> anyhow::Result<()> {
        let mut offset = 0;
        for (i, inline) in inlines.clone().into_iter().enumerate() {
            if let Inline::CodeBlock {
                source, global_idx, ..
            } = inline
            {
                let outputs = &self.outputs[global_idx];
                for output in &outputs.values {
                    match output {
                        Outval::Text(s) => {
                            let command = Command {
                                function: "output_text".to_string(),
                                id: None,
                                parameters: vec![Parameter {
                                    key: Some("value".to_string()),
                                    value: Value::String(s.clone()),
                                    pos: Default::default(),
                                }],
                                body: None,
                                pos: Default::default(),
                                global_idx: 0,
                            };

                            inlines.insert(i + offset + 1, Inline::Command(command));
                            offset += 1;
                        }
                        Outval::Image(img) => {
                            let mut params = Vec::new();
                            for (key, val) in source.meta.clone() {
                                params.push(Parameter {
                                    key: Some(key),
                                    value: Value::String(val),
                                    pos: PosInfo::new("", 0, 0),
                                });
                            }

                            match img {
                                Image::Png(png) => params.push(Parameter {
                                    key: Some("base64".to_string()),
                                    value: Value::String(png.clone()),
                                    pos: PosInfo::new("", 0, 0),
                                }),
                                Image::Svg(svg) => params.push(Parameter {
                                    key: Some("svg".to_string()),
                                    value: Value::String(svg.clone()),
                                    pos: PosInfo::new("", 0, 0),
                                }),
                            }

                            let command = Command {
                                function: "figure".to_string(),
                                id: source.meta.get("id").cloned(),
                                parameters: params,
                                body: None,
                                pos: Default::default(),
                                global_idx: 0,
                            };

                            inlines.insert(i + offset + 1, Inline::Command(command));
                            offset += 1;
                        }
                        Outval::Json(_) => {}
                        Outval::Html(_) => {}
                        Outval::Javascript(_) => {}
                        Outval::Error(_) => {}
                    }
                }
            }
        }

        self.walk_vec_inline(inlines)
    }
}

impl AstPreprocessor for CellProcessor {
    fn name(&self) -> String {
        "Cell processing".to_string()
    }

    fn process(&mut self, mut input: Document<Ast>) -> Result<Document<Ast>, Error> {
        let mut visitor = CellVisitor {
            outputs: &input.code_outputs,
        };
        visitor.walk_ast(&mut input.content.0)?;
        Ok(input)
    }
}

impl Display for CellProcessor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
