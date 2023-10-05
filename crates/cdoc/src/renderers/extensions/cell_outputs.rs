use crate::renderers::extensions::{RenderExtension, RenderExtensionConfig};
use crate::renderers::newrenderer::ElementRenderer;
use crate::renderers::Document;
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{CodeBlock, Command, Inline, Parameter, Value};
use cdoc_parser::document::{CodeOutput, Image, OutputValue};
use cdoc_parser::Span;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
//
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct CellOutputConfig;
//
// #[typetag::serde(name = "cells")]
// impl RenderExtensionConfig for CellOutputConfig {
//     fn build(&self) -> anyhow::Result<Box<dyn RenderExtension>> {
//         Ok(Box::new(CellProcessor))
//     }
// }
//
// #[derive(Debug, Default)]
// pub struct CellProcessor;
//
// pub struct CellVisitor<'a> {
//     outputs: &'a HashMap<u64, CodeOutput>,
// }
//
// impl AstVisitor for CellVisitor<'_> {
//     fn visit_vec_inline(&mut self, inlines: &mut Vec<Inline>) -> anyhow::Result<()> {
//         let mut offset = 0;
//         for (i, inline) in inlines.clone().into_iter().enumerate() {
//             if let Inline::CodeBlock(CodeBlock { source, .. }) = inline {
//                 if let Some(outputs) = self.outputs.get(&source.hash) {
//                     // println!("got output");
//                     for output in &outputs.values {
//                         match output {
//                             OutputValue::Text(s) => {
//                                 let command = Command {
//                                     function: "output_text".into(),
//                                     label: None,
//                                     parameters: vec![Parameter {
//                                         key: Some("value".into()),
//                                         value: Value::String(s.into()),
//                                         span: Default::default(),
//                                     }],
//                                     body: None,
//                                     span: Default::default(),
//                                     global_idx: 0,
//                                 };
//
//                                 inlines.insert(i + offset + 1, Inline::Command(command));
//                                 offset += 1;
//                             }
//                             OutputValue::Image(img) => {
//                                 let mut params = Vec::new();
//                                 for (key, val) in source.meta.clone() {
//                                     params.push(Parameter {
//                                         key: Some(key),
//                                         value: Value::String(val),
//                                         span: Span::new(0, 0),
//                                     });
//                                 }
//
//                                 match img {
//                                     Image::Png(png) => params.push(Parameter {
//                                         key: Some("base64".into()),
//                                         value: Value::String(png.into()),
//                                         span: Span::new(0, 0),
//                                     }),
//                                     Image::Svg(svg) => params.push(Parameter {
//                                         key: Some("svg".into()),
//                                         value: Value::String(svg.into()),
//                                         span: Span::new(0, 0),
//                                     }),
//                                 }
//
//                                 let command = Command {
//                                     function: "figure".into(),
//                                     label: source.meta.get("id").cloned(),
//                                     parameters: params,
//                                     body: None,
//                                     span: Default::default(),
//                                     global_idx: 0,
//                                 };
//
//                                 inlines.insert(i + offset + 1, Inline::Command(command));
//                                 offset += 1;
//                             }
//                             OutputValue::Json(_) => {}
//                             OutputValue::Html(_) => {}
//                             OutputValue::Javascript(_) => {}
//                             OutputValue::Error(_) => {}
//                             OutputValue::Plain(_) => {}
//                         }
//                     }
//                 }
//             }
//         }
//
//         self.walk_vec_inline(inlines)
//     }
// }
//
// impl RenderExtension for CellProcessor {
//     fn name(&self) -> String {
//         "Cell processing".to_string()
//     }
//
//     fn process(
//         &mut self,
//         ctx: &mut RenderContext,
//         _renderer: ElementRenderer,
//     ) -> anyhow::Result<()> {
//         if ctx.doc.meta.cell_outputs {
//             // Only run if outputs should be included
//             let mut visitor = CellVisitor {
//                 outputs: &ctx.doc.code_outputs,
//             };
//             visitor.walk_ast(&mut ctx.doc.content.blocks)?;
//         }
//         Ok(())
//     }
// }
//
// impl Display for CellProcessor {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.name())
//     }
// }
