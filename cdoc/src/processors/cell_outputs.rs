use crate::ast::{Ast, AstVisitor, Block, Inline, Shortcode, ShortcodeBase};
use crate::document::Document;
use crate::notebook::{CellOutput, OutputValue};
use crate::parser::ParserSettings;
use crate::parsers::shortcodes::{Argument, ArgumentValue};
use crate::processors::{AstPreprocessor, AstPreprocessorConfig, Error, PreprocessorContext};
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

impl AstVisitor for CellProcessor {
    fn visit_vec_block(&mut self, blocks: &mut Vec<Block>) -> anyhow::Result<()> {
        let mut offset = 0;
        for (i, block) in blocks.clone().into_iter().enumerate() {
            match block {
                Block::CodeBlock {
                    source,
                    reference,
                    attr,
                    tags,
                    outputs,
                    meta,
                    display_cell,
                } => {
                    for output in outputs {
                        match output {
                            CellOutput::Data {
                                execution_count,
                                data,
                                metadata,
                            } => {
                                let mut params = Vec::new();
                                for (key, val) in meta.custom.clone() {
                                    params.push(Argument::Keyword {
                                        name: key,
                                        value: ArgumentValue::Literal(vec![Block::Plain(vec![
                                            Inline::Text(val),
                                        ])]),
                                    });
                                }
                                // if let Some(cap) = meta.custom.get("caption") {
                                //     params.push(Argument::Keyword {
                                //         name: "caption".to_string(),
                                //         value: ArgumentValue::Literal(vec![Block::Plain(vec![
                                //             Inline::Text(cap.clone()),
                                //         ])]),
                                //     })
                                // };

                                let mut create_figure = false;

                                for d in data {
                                    match d {
                                        OutputValue::Plain(_) => {}
                                        OutputValue::Image(s) => {
                                            create_figure = true;
                                            params.push(Argument::Keyword {
                                                name: "base64".to_string(),
                                                value: ArgumentValue::Literal(vec![Block::Plain(
                                                    vec![Inline::Text(s)],
                                                )]),
                                            })
                                        }
                                        OutputValue::Svg(s) => {
                                            create_figure = true;
                                            params.push(Argument::Keyword {
                                                name: "svg".to_string(),
                                                value: ArgumentValue::Literal(vec![Block::Plain(
                                                    vec![Inline::Text(s)],
                                                )]),
                                            })
                                        }
                                        OutputValue::Json(_) => {}
                                        OutputValue::Html(_) => {}
                                        OutputValue::Javascript(_) => {}
                                    }
                                }

                                if create_figure {
                                    let sc = Shortcode::Inline(ShortcodeBase {
                                        name: "figure".to_string(),
                                        id: meta.custom.get("id").map(String::from),
                                        // num: 0,
                                        parameters: params,
                                        pos: Default::default(),
                                        cell: 0,
                                    });

                                    blocks.insert(
                                        i + offset + 1,
                                        Block::Plain(vec![Inline::Shortcode(sc)]),
                                    );
                                    offset += 1;
                                }
                            }
                            CellOutput::Stream { name, text } => {
                                let sc = Shortcode::Inline(ShortcodeBase {
                                    name: "output_text".to_string(),
                                    id: None,
                                    parameters: vec![
                                        Argument::Keyword {
                                            name: "stdio".to_string(),
                                            value: ArgumentValue::Literal(vec![Block::Plain(
                                                vec![Inline::Text(name.to_string())],
                                            )]),
                                        },
                                        Argument::Keyword {
                                            name: "value".to_string(),
                                            value: ArgumentValue::Literal(vec![Block::Plain(
                                                vec![Inline::Text(text)],
                                            )]),
                                        },
                                    ],
                                    pos: Default::default(),
                                    cell: 0,
                                });

                                blocks.insert(
                                    i + offset + 1,
                                    Block::Plain(vec![Inline::Shortcode(sc)]),
                                );
                                offset += 1;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        self.walk_vec_block(blocks)
    }
}

impl AstPreprocessor for CellProcessor {
    fn name(&self) -> String {
        "Cell processing".to_string()
    }

    fn process(&mut self, mut input: Document<Ast>) -> Result<Document<Ast>, Error> {
        self.walk_ast(&mut input.content)?;
        Ok(input)
    }
}

impl Display for CellProcessor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
