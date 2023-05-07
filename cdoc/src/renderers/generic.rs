use crate::ast::{Block, Inline, Shortcode};
use crate::document::Document;
use crate::notebook::{CellOutput, OutputValue, StreamType};

use crate::renderers;
use anyhow::Result;
use pulldown_cmark::HeadingLevel;

use crate::parsers::shortcodes::{ParamValue, Parameter};
use std::collections::HashMap;
use std::io::{Cursor, Write};
use tera::Context;

use crate::renderers::{
    add_args, render_basic_template, render_image, render_link, render_math, render_value_template,
    DocumentRenderer, RenderContext, RenderElement, RenderResult,
};
use crate::templates::{TemplateType, ValidationError};

// pub struct GenericRendererBuilder;
//
// impl<'a> RendererBuilder<'a> for GenericRendererBuilder {
//     type Renderer = GenericRenderer<'a>;
//
//     fn build(self, doc: &Document<Ast>) -> Self::Renderer {
//         GenericRenderer {
//             metadata: &doc.metadata,
//         }
//     }
// }

// #[derive(Serialize, Deserialize)]
pub struct GenericRenderer;

// #[typetag::serde(name = "renderer_config")]
impl DocumentRenderer for GenericRenderer {
    fn render_doc(&mut self, ctx: &RenderContext) -> Result<Document<RenderResult>> {
        // let doc = doc.to_events();
        // let dd = doc.to_events();
        //
        // let mut output = String::new();
        // html::push_html(&mut output, dd);
        let mut buf = Vec::new();
        let mut cursor = Cursor::new(buf);
        self.render(&ctx.doc.content.0, ctx, &mut cursor)?;

        let content = String::from_utf8(cursor.get_ref().clone())?;
        Ok(Document {
            content,
            metadata: ctx.doc.metadata.clone(),
            variables: ctx.doc.variables.clone(),
            ids: ctx.doc.ids.clone(),
            id_map: ctx.doc.id_map.clone(),
        })
    }
}

impl GenericRenderer {
    fn render_params(
        &mut self,
        parameters: Vec<Parameter<Vec<Block>>>,
        ctx: &RenderContext,
    ) -> Result<Vec<Parameter<String>>> {
        parameters
            .into_iter()
            .map(|p| p.try_map(|v| v.try_map(|i| Ok(self.render_inner(&i, ctx)?))))
            .collect()
    }

    fn render_shortcode_template(
        &mut self,
        ctx: &RenderContext,
        shortcode: &Shortcode,
        buf: impl Write,
    ) -> Result<()> {
        let mut args = ctx.extra_args.clone();
        args.insert("defs", &ctx.templates.definitions);

        let name = match shortcode {
            Shortcode::Inline(def) => {
                let rendered = self.render_params(def.parameters.clone(), ctx)?;
                let r: anyhow::Result<Vec<()>> = ctx
                    .templates
                    .validate_args_for_template(&def.name, &rendered)?
                    .into_iter()
                    .collect();
                r?;
                add_args(
                    &mut args,
                    &def.id,
                    def.num,
                    &ctx.ids,
                    &ctx.ids_map,
                    rendered,
                )?;
                def.name.clone()
            }
            Shortcode::Block(def, body) => {
                let rendered = self.render_params(def.parameters.clone(), ctx)?;
                let r: anyhow::Result<Vec<()>> = ctx
                    .templates
                    .validate_args_for_template(&def.name, &rendered)?
                    .into_iter()
                    .collect();
                r?;
                add_args(
                    &mut args,
                    &def.id,
                    def.num,
                    &ctx.ids,
                    &ctx.ids_map,
                    rendered,
                )?;
                let body = self.render_inner(body, ctx)?;
                args.insert("body", &body);
                def.name.clone()
            }
        };
        ctx.templates
            .render(&name, ctx.format, TemplateType::Shortcode, &args, buf)
    }
}

// pub struct ToHtmlContext<'a> {
//     pub metadata: DocumentMetadata,
//     pub ids: HashMap<String, (usize, Vec<ShortCodeDef>)>,
//     pub ids_map: HashMap<String, (usize, ShortCodeDef)>,
//     pub templates: &'a TemplateManager,
//     pub extra_args: Context,
//     pub syntax_set: SyntaxSet,
//     pub theme: Theme,
// }
//
// pub trait ToHtml {
//     fn to_html(self, ctx: &ToHtmlContext) -> Result<String>;
// }

impl RenderElement<Inline> for GenericRenderer {
    fn render(&mut self, elem: &Inline, ctx: &RenderContext, mut buf: impl Write) -> Result<()> {
        match elem {
            Inline::Text(s) => {
                buf.write(s.as_bytes())?;
                Ok(())
            }
            Inline::Emphasis(inner) => render_value_template(
                "emphasis",
                TemplateType::Builtin,
                &self.render_inner(inner, ctx)?,
                ctx,
                buf,
            ),
            Inline::Strong(inner) => render_value_template(
                "strong",
                TemplateType::Builtin,
                &self.render_inner(inner, ctx)?,
                ctx,
                buf,
            ),
            Inline::Strikethrough(inner) => render_value_template(
                "strikethrough",
                TemplateType::Builtin,
                &self.render_inner(inner, ctx)?,
                ctx,
                buf,
            ),
            Inline::Code(s) => {
                render_value_template("inline_code", TemplateType::Builtin, s, ctx, buf)
            }
            Inline::SoftBreak => {
                render_basic_template("soft_break", TemplateType::Builtin, ctx, buf)
            }
            Inline::HardBreak => {
                render_basic_template("hard_break", TemplateType::Builtin, ctx, buf)
            }
            Inline::Rule => {
                render_basic_template("horizontal_rule", TemplateType::Builtin, ctx, buf)
            }
            Inline::Image(_tp, url, alt, inner) => {
                let inner = self.render_inner(inner, ctx)?;
                render_image(url, alt, &inner, ctx, buf)
            }
            Inline::Link(_tp, url, alt, inner) => {
                let inner = self.render_inner(inner, ctx)?;
                render_link(url, alt, &inner, ctx, buf)
            }
            Inline::Html(s) => {
                buf.write(s.as_bytes())?;
                Ok(())
            }
            Inline::Math(s, display_mode, trailing_space) => {
                render_math(*display_mode, *trailing_space, s, ctx, buf)
            }
            Inline::Shortcode(s) => Ok(self.render_shortcode_template(ctx, s, buf)?),
        }
    }
}

// impl RenderElement for Inline {
//     fn render(&mut self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<String> {
//         match self {
//             Inline::Text(s) => Ok(s),
//             Inline::Emphasis(inner) => Ok(format!("<em>{}</em>", inner.render(doc, ctx)?)),
//             Inline::Strong(inner) => Ok(format!("<strong>{}</strong>", inner.render(doc, ctx)?)),
//             Inline::Strikethrough(inner) => Ok(format!("<s>{}</s>", inner.render(doc, ctx)?)),
//             Inline::Code(s) => Ok(format!("<code>{}</code>", s)),
//             Inline::SoftBreak => Ok("<br>".to_string()),
//             Inline::HardBreak => Ok("<br>".to_string()),
//             Inline::Rule => Ok("<hr>".to_string()),
//             Inline::Image(_tp, url, alt, inner) => {
//                 let inner_s = inner.to_html(ctx)?;
//                 render_image(&url, &alt, &inner_s, &ctx.templates, OutputFormat::Html)
//             }
//             Inline::Link(_tp, url, alt, inner) => {
//                 let inner_s = inner.to_html(ctx)?;
//                 render_link(&url, &alt, &inner_s, &ctx.templates, OutputFormat::Html)
//             }
//             Inline::Html(s) => Ok(s),
//             Inline::Math(s, display_mode, trailing_space) => render_math(
//                 *display_mode,
//                 *trailing_space,
//                 s,
//                 ctx.templates,
//                 OutputFormat::Html,
//             ),
//             Inline::Shortcode(s) => {
//                 Ok(render_shortcode_template(ctx, s).unwrap_or_else(|e| e.to_string()))
//             }
//         }
//     }
// }
//

impl RenderElement<OutputValue> for GenericRenderer {
    fn render(
        &mut self,
        elem: &OutputValue,
        ctx: &RenderContext,
        mut buf: impl Write,
    ) -> Result<()> {
        match elem {
            OutputValue::Plain(s) => {
                render_value_template("output_text", TemplateType::Builtin, &s.join(""), ctx, buf)
            }
            OutputValue::Image(s) => {
                renderers::render_value_template("output_img", TemplateType::Builtin, s, ctx, buf)
            }
            OutputValue::Svg(s) => {
                renderers::render_value_template("output_svg", TemplateType::Builtin, s, ctx, buf)
            }
            OutputValue::Json(s) => {
                buf.write(serde_json::to_string(&s)?.as_bytes())?;
                Ok(())
            }
            OutputValue::Html(s) => {
                buf.write(s.as_bytes())?;
                Ok(())
            }
            OutputValue::Javascript(_) => Ok(()),
        }
    }
}

impl RenderElement<CellOutput> for GenericRenderer {
    fn render(
        &mut self,
        elem: &CellOutput,
        ctx: &RenderContext,
        mut buf: impl Write,
    ) -> Result<()> {
        match elem {
            CellOutput::Stream { text, name } => match name {
                StreamType::StdOut => {
                    render_value_template("output_text", TemplateType::Builtin, text, ctx, buf)
                }
                StreamType::StdErr => {
                    render_value_template("output_error", TemplateType::Builtin, text, ctx, buf)
                }
            },
            CellOutput::Data { data, .. } => {
                for v in data {
                    self.render(v, ctx, &mut buf)?;
                }
                Ok(())
            }
            CellOutput::Error { evalue, .. } => {
                render_value_template("output_error", TemplateType::Builtin, evalue, ctx, buf)
            }
        }
    }
}

pub fn header_lvl_to_int(lvl: &HeadingLevel) -> usize {
    match lvl {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

impl RenderElement<Block> for GenericRenderer {
    fn render(&mut self, elem: &Block, ctx: &RenderContext, buf: impl Write) -> Result<()> {
        match elem {
            Block::Heading { lvl, inner, .. } => {
                let mut args = Context::default();
                // println!("{}", );
                args.insert("level", &header_lvl_to_int(lvl));
                args.insert("inner", &self.render_inner(inner, ctx)?);
                Ok(ctx
                    .templates
                    .render("header", ctx.format, TemplateType::Builtin, &args, buf)?)
            }
            Block::Plain(inner) => self.render(inner, ctx, buf),
            Block::Paragraph(inner) | Block::BlockQuote(inner) => render_value_template(
                "paragraph",
                TemplateType::Builtin,
                &self.render_inner(inner, ctx)?,
                ctx,
                buf,
            ),
            Block::CodeBlock {
                source,
                outputs,
                tags,
                ..
            } => {
                let id = renderers::get_id();

                let highlighted = syntect::html::highlighted_html_for_string(
                    source,
                    &ctx.syntax_set,
                    ctx.syntax_set.find_syntax_by_extension("py").unwrap(),
                    &ctx.theme,
                )?;

                let mut args = Context::default();
                args.insert("interactive", &ctx.doc.metadata.interactive);
                args.insert("cell_outputs", &ctx.doc.metadata.cell_outputs);
                args.insert("editable", &ctx.doc.metadata.editable);
                args.insert("source", &source);
                args.insert("highlighted", &highlighted);
                args.insert("id", &id);
                args.insert("tags", &tags);
                args.insert("outputs", &self.render_inner(outputs, ctx)?);

                Ok(ctx
                    .templates
                    .render("cell", ctx.format, TemplateType::Builtin, &args, buf)?)
            }
            Block::List(idx, items) => {
                let inner = self.render_inner(items, ctx)?;
                // let inner: Result<String> = items.iter().map(|b| self.render(b, ctx)).collect();
                // let inner = inner?;

                Ok(match idx {
                    None => render_value_template(
                        "list_unordered",
                        TemplateType::Builtin,
                        &inner,
                        ctx,
                        buf,
                    )?,
                    Some(start) => {
                        let mut args = Context::default();
                        args.insert("start", &start);
                        args.insert("value", &inner);
                        ctx.templates.render(
                            "list_ordered",
                            ctx.format,
                            TemplateType::Builtin,
                            &args,
                            buf,
                        )?
                    }
                })
            }
            Block::ListItem(inner) => render_value_template(
                "list_item",
                TemplateType::Builtin,
                &self.render_inner(inner, ctx)?,
                ctx,
                buf,
            ),
        }
    }
}
