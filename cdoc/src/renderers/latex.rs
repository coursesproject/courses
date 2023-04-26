use crate::ast::{Ast, Block, Inline, Shortcode};
use crate::document::{Document, DocumentMetadata};
use crate::notebook::{CellOutput, OutputValue};
use crate::parsers::shortcodes::ShortCodeDef;
use crate::renderers::{
    add_args, get_id, render_value_template, RenderContext, RenderElement, RenderResult, Renderer,
};
use anyhow::Result;
use pulldown_cmark::HeadingLevel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tera::Tera;

#[derive(Serialize, Deserialize)]
pub struct LatexRenderer;

#[typetag::serde(name = "renderer_config")]
impl Renderer for LatexRenderer {
    fn render(&self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<Document<RenderResult>> {
        let ctx = ToLaTeXContext {
            metadata: doc.metadata.clone(),
            ids: doc.ids.clone(),
            ids_map: doc.id_map.clone(),
            tera: ctx.tera.clone(),
            tera_context: ctx.tera_context.clone(),
        };

        Ok(Document {
            content: doc.content.0.clone().render(&ctx)?,
            metadata: doc.metadata.clone(),
            ids: doc.ids.clone(),
            id_map: doc.id_map.clone(),
            variables: doc.variables.clone(),
        })
    }
}

pub struct ToLaTeXContext {
    pub metadata: DocumentMetadata,
    pub ids: HashMap<String, (usize, Vec<ShortCodeDef>)>,
    pub ids_map: HashMap<String, (usize, ShortCodeDef)>,
    pub tera: Tera,
    pub tera_context: tera::Context,
}

impl RenderElement<ToLaTeXContext> for Inline {
    fn render(self, ctx: &ToLaTeXContext) -> Result<String> {
        match self {
            Inline::Text(s) => Ok(s),
            Inline::Emphasis(inner) => {
                let r = inner.render(ctx)?;
                Ok(format!("\\emph{{{r}}}"))
            }
            Inline::Strong(inner) | Inline::Strikethrough(inner) => {
                let r = inner.render(ctx)?;
                Ok(format!("\\textbf{{{r}}}"))
            }
            Inline::Code(s) => Ok(format!("\\lstinline! {s} !")),
            Inline::SoftBreak => Ok("\n".to_string()),
            Inline::HardBreak => Ok("\n\\\\\n".to_string()),
            Inline::Rule => Ok("\\hrule".to_string()),
            Inline::Image(_tp, url, alt, inner) => {
                let inner_s = inner.render(ctx)?;
                let mut context = tera::Context::new();
                context.insert("url", &url);
                context.insert("alt", &alt);
                context.insert("inner", &inner_s);
                Ok(ctx.tera.render("builtins/latex/image.tera.tex", &context)?)
            }
            Inline::Link(_tp, url, alt, inner) => {
                let inner_s = inner.render(ctx)?;
                let mut context = tera::Context::new();
                context.insert("url", &url);
                context.insert("alt", &alt);
                context.insert("inner", &inner_s);
                Ok(ctx.tera.render("builtins/latex/link.tera.tex", &context)?)
            }
            Inline::Html(s) => Ok(s),
            Inline::Math(s, display_mode, trailing_space) => {
                let mut context = tera::Context::new();
                context.insert("display_mode", &display_mode);
                context.insert("trailing_space", &trailing_space);
                context.insert("value", &s);
                Ok(ctx.tera.render("builtins/latex/math.tera.tex", &context)?)
            }
            Inline::Shortcode(s) => render_shortcode_template(ctx, s),
        }
    }
}

impl RenderElement<ToLaTeXContext> for OutputValue {
    fn render(self, ctx: &ToLaTeXContext) -> Result<String> {
        match self {
            OutputValue::Plain(s) => {
                render_value_template(&ctx.tera, "builtins/latex/output_text.tera.tex", s.join(""))
            }
            OutputValue::Image(s) => {
                render_value_template(&ctx.tera, "builtins/latex/output_img.tera.tex", s)
            }
            OutputValue::Svg(s) => {
                render_value_template(&ctx.tera, "builtins/latex/output_svg.tera.tex", s)
            }
            OutputValue::Json(_) => Ok("".to_string()),
            OutputValue::Html(_) => Ok("".to_string()),
            OutputValue::Javascript(_) => Ok("".to_string()),
        }
    }
}

impl RenderElement<ToLaTeXContext> for CellOutput {
    fn render(self, ctx: &ToLaTeXContext) -> Result<String> {
        match self {
            CellOutput::Stream { text, .. } => Ok(text),
            CellOutput::Data { data, .. } => data.into_iter().map(|v| v.render(ctx)).collect(),
            CellOutput::Error { evalue, .. } => {
                render_value_template(&ctx.tera, "builtins/latex/output_error.tera.md", evalue)
            }
        }
    }
}

impl RenderElement<ToLaTeXContext> for Block {
    fn render(self, ctx: &ToLaTeXContext) -> Result<String> {
        match self {
            Block::Heading { lvl, inner, .. } => {
                let cmd = match lvl {
                    HeadingLevel::H1 => "section",
                    HeadingLevel::H2 => "subsection",
                    _ => "subsubsection",
                };
                Ok(format!("\\{cmd}{{{}}}\n", inner.render(ctx)?))
            }
            Block::Plain(inner) => inner.render(ctx),
            Block::Paragraph(inner) | Block::BlockQuote(inner) => {
                Ok(format!("{}\n", inner.render(ctx)?))
            }
            Block::CodeBlock {
                source,
                outputs,
                tags,
                ..
            } => {
                let id = get_id();

                let mut context = tera::Context::new();
                context.insert("cell_outputs", &ctx.metadata.cell_outputs);
                context.insert("source", &source);
                context.insert("id", &id);
                context.insert("tags", &tags);
                context.insert("outputs", &outputs.render(ctx)?);

                let output = ctx.tera.render("builtins/latex/cell.tera.tex", &context)?;
                Ok(output)
            }
            Block::List(idx, items) => {
                let inner: Result<String> = items.into_iter().map(|b| b.render(ctx)).collect();
                let inner = inner?;

                Ok(match idx {
                    None => render_value_template(
                        &ctx.tera,
                        "builtins/latex/list_unordered.tera.tex",
                        inner,
                    )?,
                    Some(start) => {
                        let mut context = tera::Context::new();
                        context.insert("start", &start);
                        context.insert("value", &inner);
                        ctx.tera
                            .render("builtins/latex/list_ordered.tera.tex", &context)?
                    }
                })
            }
            Block::ListItem(inner) => render_value_template(
                &ctx.tera,
                "builtins/latex/list_item.tera.tex",
                inner.render(ctx)?,
            ),
        }
    }
}

fn render_params(
    parameters: HashMap<String, Vec<Block>>,
    ctx: &ToLaTeXContext,
) -> Result<HashMap<String, String>> {
    parameters
        .into_iter()
        .map(|(k, v)| Ok((k, v.render(ctx)?)))
        .collect()
}

fn render_shortcode_template(ctx: &ToLaTeXContext, shortcode: Shortcode) -> Result<String> {
    let mut context = ctx.tera_context.clone();

    match shortcode {
        Shortcode::Inline(def) => {
            let name = format!("shortcodes/tex/{}.tera.tex", def.name,);
            add_args(
                &mut context,
                def.id,
                def.num,
                &ctx.ids,
                &ctx.ids_map,
                render_params(def.parameters, ctx)?,
            );
            Ok(ctx.tera.render(&name, &context)?)
        }
        Shortcode::Block(def, body) => {
            let name = format!("shortcodes/tex/{}.tera.tex", def.name,);
            add_args(
                &mut context,
                def.id,
                def.num,
                &ctx.ids,
                &ctx.ids_map,
                render_params(def.parameters, ctx)?,
            );
            let body = body.render(ctx)?;
            context.insert("body", &body);
            Ok(ctx.tera.render(&name, &context)?)
        }
    }
}
