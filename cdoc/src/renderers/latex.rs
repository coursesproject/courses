// use crate::ast::{Ast, Block, Inline, Shortcode};
// use crate::config::OutputFormat;
// use crate::document::{Document, DocumentMetadata};
// use crate::notebook::{CellOutput, OutputValue};
// use crate::parsers::shortcodes::ShortCodeDef;
// use crate::renderers::{
//     add_args, get_id, render_image, render_link, render_math, render_value_template,
//     DocumentRenderer, RenderContext, RenderElement, RenderResult,
// };
// use crate::templates::{TemplateContext, TemplateManager, TemplateType};
// use anyhow::Result;
// use pulldown_cmark::HeadingLevel;
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use std::collections::{BTreeMap, HashMap};
// use std::rc::Rc;
// use std::sync::Arc;
// use tera::Tera;
//
// #[derive(Serialize, Deserialize)]
// pub struct LatexRenderer;
//
// #[typetag::serde(name = "renderer_config")]
// impl DocumentRenderer for LatexRenderer {
//     fn render(&self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<Document<RenderResult>> {
//         Ok(Document {
//             content: doc.content.0.clone().render(doc, ctx)?,
//             metadata: doc.metadata.clone(),
//             ids: doc.ids.clone(),
//             id_map: doc.id_map.clone(),
//             variables: doc.variables.clone(),
//         })
//     }
// }
//
// pub struct ToLaTeXContext<'a> {
//     pub metadata: DocumentMetadata,
//     pub ids: HashMap<String, (usize, Vec<ShortCodeDef>)>,
//     pub ids_map: HashMap<String, (usize, ShortCodeDef)>,
//     pub templates: &'a TemplateManager,
//     pub extra_args: TemplateContext,
// }
//
// impl RenderElement for Inline {
//     fn render(&mut self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<String> {
//         match self {
//             Inline::Text(s) => Ok(s.to_string()),
//             Inline::Emphasis(inner) => {
//                 let r = inner.render(doc, ctx)?;
//                 Ok(format!("\\emph{{{r}}}"))
//             }
//             Inline::Strong(inner) | Inline::Strikethrough(inner) => {
//                 let r = inner.render(doc, ctx)?;
//                 Ok(format!("\\textbf{{{r}}}"))
//             }
//             Inline::Code(s) => Ok(format!("\\lstinline! {s} !")),
//             Inline::SoftBreak => Ok("\n".to_string()),
//             Inline::HardBreak => Ok("\n\\\\\n".to_string()),
//             Inline::Rule => Ok("\\hrule".to_string()),
//             Inline::Image(_tp, url, alt, inner) => {
//                 let inner_s = inner.render(doc, ctx)?;
//                 render_image(&url, &alt, &inner_s, &ctx.templates, OutputFormat::LaTeX)
//             }
//             Inline::Link(_tp, url, alt, inner) => {
//                 let inner_s = inner.render(doc, ctx)?;
//                 render_link(&url, &alt, &inner_s, &ctx.templates, OutputFormat::Html)
//             }
//             Inline::Html(s) => Ok(s.to_string()),
//             Inline::Math(s, display_mode, trailing_space) => render_math(
//                 *display_mode,
//                 *trailing_space,
//                 s,
//                 ctx.templates,
//                 OutputFormat::Html,
//             ),
//             Inline::Shortcode(s) => render_shortcode_template(doc, ctx, s),
//         }
//     }
// }
//
// impl RenderElement for OutputValue {
//     fn render(&mut self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<String> {
//         match self {
//             OutputValue::Plain(s) => render_value_template(
//                 &ctx.templates,
//                 "output_text",
//                 OutputFormat::LaTeX,
//                 TemplateType::Builtin,
//                 s.join(""),
//             ),
//             OutputValue::Image(s) => render_value_template(
//                 &ctx.templates,
//                 "output_img",
//                 OutputFormat::LaTeX,
//                 TemplateType::Builtin,
//                 s,
//             ),
//             OutputValue::Svg(s) => render_value_template(
//                 &ctx.templates,
//                 "output_svg",
//                 OutputFormat::LaTeX,
//                 TemplateType::Builtin,
//                 s,
//             ),
//             OutputValue::Json(_) => Ok("".to_string()),
//             OutputValue::Html(_) => Ok("".to_string()),
//             OutputValue::Javascript(_) => Ok("".to_string()),
//         }
//     }
// }
//
// impl RenderElement for CellOutput {
//     fn render(&mut self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<String> {
//         match self {
//             CellOutput::Stream { text, .. } => Ok(text),
//             CellOutput::Data { data, .. } => data.into_iter().map(|v| v.render(doc, ctx)).collect(),
//             CellOutput::Error { evalue, .. } => render_value_template(
//                 &ctx.templates,
//                 "output_error",
//                 OutputFormat::LaTeX,
//                 TemplateType::Builtin,
//                 evalue,
//             ),
//         }
//     }
// }
//
// impl RenderElement for Block {
//     fn render(&mut self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<String> {
//         match self {
//             Block::Heading { lvl, inner, .. } => {
//                 let cmd = match lvl {
//                     HeadingLevel::H1 => "section",
//                     HeadingLevel::H2 => "subsection",
//                     _ => "subsubsection",
//                 };
//                 Ok(format!("\\{cmd}{{{}}}\n", inner.render(doc, ctx)?))
//             }
//             Block::Plain(inner) => inner.render(doc, ctx),
//             Block::Paragraph(inner) | Block::BlockQuote(inner) => {
//                 Ok(format!("{}\n", inner.render(doc, ctx)?))
//             }
//             Block::CodeBlock {
//                 source,
//                 outputs,
//                 tags,
//                 ..
//             } => {
//                 let id = get_id();
//
//                 let mut args = TemplateContext::new();
//                 args.insert("cell_outputs", &ctx.metadata.cell_outputs);
//                 args.insert("source", &source);
//                 args.insert("id", &id);
//                 args.insert("tags", &tags);
//                 args.insert("outputs", &outputs.render(doc, ctx)?);
//
//                 let output = ctx.templates.render(
//                     "cell",
//                     OutputFormat::LaTeX,
//                     TemplateType::Builtin,
//                     &args,
//                 )?;
//                 Ok(output)
//             }
//             Block::List(idx, items) => {
//                 let inner: Result<String> = items.into_iter().map(|b| b.render(doc, ctx)).collect();
//                 let inner = inner?;
//
//                 Ok(match idx {
//                     None => render_value_template(
//                         &ctx.templates,
//                         "list_unordered",
//                         OutputFormat::LaTeX,
//                         TemplateType::Builtin,
//                         inner,
//                     )?,
//                     Some(start) => {
//                         let mut args = TemplateContext::new();
//                         args.insert("start", &start);
//                         args.insert("value", &inner);
//                         ctx.templates.render(
//                             "list_ordered",
//                             OutputFormat::LaTeX,
//                             TemplateType::Builtin,
//                             &args,
//                         )?
//                     }
//                 })
//             }
//             Block::ListItem(inner) => render_value_template(
//                 &ctx.templates,
//                 "list_item",
//                 OutputFormat::LaTeX,
//                 TemplateType::Builtin,
//                 inner.render(doc, ctx)?,
//             ),
//         }
//     }
// }
//
// fn render_params(
//     parameters: &mut HashMap<String, Vec<Block>>,
//     doc: &Document<Ast>,
//     ctx: &RenderContext,
// ) -> Result<HashMap<String, String>> {
//     parameters
//         .iter_mut()
//         .map(|(k, v)| Ok((k, v.render(doc, ctx)?)))
//         .collect()
// }
//
// fn render_shortcode_template(
//     doc: &Document<Ast>,
//     ctx: &RenderContext,
//     shortcode: &Shortcode,
// ) -> Result<String> {
//     let mut args = ctx.extra_args.clone();
//
//     let name = match shortcode {
//         Shortcode::Inline(def) => {
//             add_args(
//                 &mut args,
//                 def.id,
//                 def.num,
//                 &ctx.ids,
//                 &ctx.ids_map,
//                 render_params(def.parameters, ctx)?,
//             );
//             &def.name
//         }
//         Shortcode::Block(def, mut body) => {
//             let name = format!("shortcodes/tex/{}.tera.tex", def.name,);
//             add_args(
//                 &mut args,
//                 def.id,
//                 def.num,
//                 &ctx.ids,
//                 &ctx.ids_map,
//                 render_params(def.parameters, ctx)?,
//             );
//             let body = body.render(doc, ctx)?;
//             args.insert("body", &body);
//             &def.name
//         }
//     };
//     Ok(ctx
//         .templates
//         .render(name, OutputFormat::LaTeX, TemplateType::Shortcode, &args)?)
// }
