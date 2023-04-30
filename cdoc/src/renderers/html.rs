// use crate::ast::{Ast, Block, Inline, Shortcode};
// use crate::config::{HtmlFormat, OutputFormat};
// use crate::document::{Document, DocumentMetadata};
// use crate::notebook::{CellOutput, OutputValue, StreamType};
// use crate::parsers::shortcodes::ShortCodeDef;
// use crate::renderers;
// use anyhow::Result;
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use std::collections::{BTreeMap, HashMap};
// use syntect::highlighting::Theme;
// use syntect::parsing::SyntaxSet;
// use tera::Tera;
//
// use crate::renderers::{
//     add_args, render_image, render_link, render_math, render_shortcode_template, DocumentRenderer,
//     RenderContext, RenderElement, RenderResult,
// };
// use crate::templates::{TemplateContext, TemplateManager, TemplateType};
//
// #[derive(Serialize, Deserialize)]
// pub struct HtmlRenderer {
//     pub(crate) interactive_cells: bool,
//     pub metadata: DocumentMetadata,
// }
//
// #[typetag::serde(name = "renderer_config")]
// impl DocumentRenderer for HtmlRenderer {
//     fn render_doc(
//         &mut self,
//         doc: &Document<Ast>,
//         ctx: &RenderContext,
//     ) -> Result<Document<RenderResult>> {
//         // let doc = doc.to_events();
//         // let dd = doc.to_events();
//         //
//         // let mut output = String::new();
//         // html::push_html(&mut output, dd);
//
//         let content = self.render(&doc.content.0, ctx)?;
//         Ok(Document {
//             content,
//             metadata: doc.metadata.clone(),
//             variables: doc.variables.clone(),
//             ids: doc.ids.clone(),
//             id_map: doc.id_map.clone(),
//         })
//     }
// }
//
// // pub struct ToHtmlContext<'a> {
// //     pub metadata: DocumentMetadata,
// //     pub ids: HashMap<String, (usize, Vec<ShortCodeDef>)>,
// //     pub ids_map: HashMap<String, (usize, ShortCodeDef)>,
// //     pub templates: &'a TemplateManager,
// //     pub extra_args: TemplateContext,
// //     pub syntax_set: SyntaxSet,
// //     pub theme: Theme,
// // }
// //
// // pub trait ToHtml {
// //     fn to_html(self, ctx: &ToHtmlContext) -> Result<String>;
// // }
//
// impl RenderElement<Inline> for HtmlRenderer {
//     fn render(&mut self, elem: &Inline, ctx: &RenderContext) -> Result<String> {
//         match elem {
//             Inline::Text(s) => Ok(s.to_string()),
//             Inline::Emphasis(inner) => Ok(format!("<em>{}</em>", self.render(inner, ctx)?)),
//             Inline::Strong(inner) => Ok(format!("<strong>{}</strong>", self.render(inner, ctx)?)),
//             Inline::Strikethrough(inner) => Ok(format!("<s>{}</s>", self.render(inner, ctx)?)),
//             Inline::Code(s) => Ok(format!("<code>{}</code>", s)),
//             Inline::SoftBreak => Ok("<br>".to_string()),
//             Inline::HardBreak => Ok("<br>".to_string()),
//             Inline::Rule => Ok("<hr>".to_string()),
//             Inline::Image(_tp, url, alt, inner) => {
//                 let inner_s = self.render(inner, ctx)?;
//                 render_image(&url, &alt, &inner_s, &ctx.templates, OutputFormat::Html)
//             }
//             Inline::Link(_tp, url, alt, inner) => {
//                 let inner_s = self.render(inner, ctx)?;
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
//             Inline::Shortcode(s) => {
//                 // Ok(render_shortcode_template(ctx, s).unwrap_or_else(|e| e.to_string()))
//                 Ok("".to_string())
//             }
//         }
//     }
// }
//
// // impl RenderElement for Inline {
// //     fn render(&mut self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<String> {
// //         match self {
// //             Inline::Text(s) => Ok(s),
// //             Inline::Emphasis(inner) => Ok(format!("<em>{}</em>", inner.render(doc, ctx)?)),
// //             Inline::Strong(inner) => Ok(format!("<strong>{}</strong>", inner.render(doc, ctx)?)),
// //             Inline::Strikethrough(inner) => Ok(format!("<s>{}</s>", inner.render(doc, ctx)?)),
// //             Inline::Code(s) => Ok(format!("<code>{}</code>", s)),
// //             Inline::SoftBreak => Ok("<br>".to_string()),
// //             Inline::HardBreak => Ok("<br>".to_string()),
// //             Inline::Rule => Ok("<hr>".to_string()),
// //             Inline::Image(_tp, url, alt, inner) => {
// //                 let inner_s = inner.to_html(ctx)?;
// //                 render_image(&url, &alt, &inner_s, &ctx.templates, OutputFormat::Html)
// //             }
// //             Inline::Link(_tp, url, alt, inner) => {
// //                 let inner_s = inner.to_html(ctx)?;
// //                 render_link(&url, &alt, &inner_s, &ctx.templates, OutputFormat::Html)
// //             }
// //             Inline::Html(s) => Ok(s),
// //             Inline::Math(s, display_mode, trailing_space) => render_math(
// //                 *display_mode,
// //                 *trailing_space,
// //                 s,
// //                 ctx.templates,
// //                 OutputFormat::Html,
// //             ),
// //             Inline::Shortcode(s) => {
// //                 Ok(render_shortcode_template(ctx, s).unwrap_or_else(|e| e.to_string()))
// //             }
// //         }
// //     }
// // }
// //
//
// impl RenderElement<OutputValue> for HtmlRenderer {
//     fn render(&mut self, elem: &OutputValue, ctx: &RenderContext) -> Result<String> {
//         match elem {
//             OutputValue::Plain(s) => renderers::render_value_template(
//                 &ctx.templates,
//                 "output_text",
//                 OutputFormat::Html,
//                 TemplateType::Builtin,
//                 &s.join(""),
//             ),
//             OutputValue::Image(s) => renderers::render_value_template(
//                 &ctx.templates,
//                 "output_img",
//                 OutputFormat::Html,
//                 TemplateType::Builtin,
//                 s,
//             ),
//             OutputValue::Svg(s) => renderers::render_value_template(
//                 &ctx.templates,
//                 "output_svg",
//                 OutputFormat::Html,
//                 TemplateType::Builtin,
//                 s,
//             ),
//             OutputValue::Json(s) => Ok(serde_json::to_string(&s)?),
//             OutputValue::Html(s) => Ok(s.to_string()),
//             OutputValue::Javascript(_) => Ok("".to_string()),
//         }
//     }
// }
//
// impl RenderElement<CellOutput> for HtmlRenderer {
//     fn render(&mut self, elem: &CellOutput, ctx: &RenderContext) -> Result<String> {
//         match elem {
//             CellOutput::Stream { text, name } => match name {
//                 StreamType::StdOut => renderers::render_value_template(
//                     &ctx.templates,
//                     "output_text",
//                     OutputFormat::Html,
//                     TemplateType::Builtin,
//                     text,
//                 ),
//                 StreamType::StdErr => renderers::render_value_template(
//                     &ctx.templates,
//                     "output_error",
//                     OutputFormat::Html,
//                     TemplateType::Builtin,
//                     text,
//                 ),
//             },
//             CellOutput::Data { data, .. } => {
//                 data.into_iter().map(|v| self.render(v, ctx)).collect()
//             }
//             CellOutput::Error { evalue, .. } => renderers::render_value_template(
//                 &ctx.templates,
//                 "output_error",
//                 OutputFormat::Html,
//                 TemplateType::Builtin,
//                 evalue,
//             ),
//         }
//     }
// }
//
// impl RenderElement<Block> for HtmlRenderer {
//     fn render(&mut self, elem: &Block, ctx: &RenderContext) -> Result<String> {
//         match elem {
//             Block::Heading { lvl, inner, .. } => {
//                 Ok(format!("<{lvl}>{}</{lvl}>", self.render(inner, ctx)?))
//             }
//             Block::Plain(inner) => self.render(inner, ctx),
//             Block::Paragraph(inner) | Block::BlockQuote(inner) => self.render(inner, ctx),
//             Block::CodeBlock {
//                 source,
//                 outputs,
//                 tags,
//                 ..
//             } => {
//                 let id = renderers::get_id();
//
//                 let highlighted = syntect::html::highlighted_html_for_string(
//                     &source,
//                     &ctx.syntax_set,
//                     ctx.syntax_set.find_syntax_by_extension("py").unwrap(),
//                     &ctx.theme,
//                 )?;
//
//                 let mut args = TemplateContext::new();
//                 args.insert("interactive", &self.metadata.interactive);
//                 args.insert("cell_outputs", &self.metadata.cell_outputs);
//                 args.insert("editable", &self.metadata.editable);
//                 args.insert("source", &source);
//                 args.insert("highlighted", &highlighted);
//                 args.insert("id", &id);
//                 args.insert("tags", &tags);
//                 args.insert("outputs", &self.render(outputs, ctx)?);
//
//                 Ok(ctx.templates.render(
//                     "cell",
//                     OutputFormat::Html,
//                     TemplateType::Builtin,
//                     &args,
//                 )?)
//             }
//             Block::List(idx, items) => {
//                 let inner: Result<String> =
//                     items.into_iter().map(|b| self.render(b, ctx)).collect();
//                 let inner = inner?;
//
//                 Ok(match idx {
//                     None => renderers::render_value_template(
//                         &ctx.templates,
//                         "list_unordered",
//                         OutputFormat::Html,
//                         TemplateType::Builtin,
//                         &inner,
//                     )?,
//                     Some(start) => {
//                         let mut args = TemplateContext::new();
//                         args.insert("start", &start);
//                         args.insert("value", &inner);
//                         ctx.templates.render(
//                             "list_ordered",
//                             OutputFormat::Html,
//                             TemplateType::Builtin,
//                             &args,
//                         )?
//                     }
//                 })
//             }
//             Block::ListItem(inner) => renderers::render_value_template(
//                 &ctx.templates,
//                 "list_item",
//                 OutputFormat::Html,
//                 TemplateType::Builtin,
//                 &self.render(inner, ctx)?,
//             ),
//         }
//     }
// }
