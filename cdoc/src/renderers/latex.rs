use crate::ast::{Ast, Block, Inline, Shortcode};
use crate::config::OutputFormat;
use crate::document::{Document, DocumentMetadata};
use crate::notebook::{CellOutput, OutputValue};
use crate::parsers::shortcodes::ShortCodeDef;
use crate::renderers::{
    add_args, get_id, render_image, render_link, render_math, render_value_template, RenderContext,
    RenderElement, RenderResult, Renderer,
};
use crate::templates::{TemplateContext, TemplateManager};
use anyhow::Result;
use pulldown_cmark::HeadingLevel;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
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
            templates: ctx.templates.clone(),
            extra_args: ctx.extra_args.clone(),
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
    pub templates: TemplateManager,
    pub extra_args: TemplateContext,
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
                render_image(&url, &alt, &inner_s, &ctx.templates, OutputFormat::LaTeX)
            }
            Inline::Link(_tp, url, alt, inner) => {
                let inner_s = inner.render(ctx)?;
                render_link(&url, &alt, &inner_s, &ctx.templates, OutputFormat::Html)
            }
            Inline::Html(s) => Ok(s),
            Inline::Math(s, display_mode, trailing_space) => render_math(
                display_mode,
                trailing_space,
                &s,
                &ctx.templates,
                OutputFormat::Html,
            ),
            Inline::Shortcode(s) => render_shortcode_template(ctx, s),
        }
    }
}

impl RenderElement<ToLaTeXContext> for OutputValue {
    fn render(self, ctx: &ToLaTeXContext) -> Result<String> {
        match self {
            OutputValue::Plain(s) => render_value_template(
                &ctx.templates,
                "b_output_text",
                OutputFormat::LaTeX,
                s.join(""),
            ),
            OutputValue::Image(s) => {
                render_value_template(&ctx.templates, "b_output_img", OutputFormat::LaTeX, s)
            }
            OutputValue::Svg(s) => {
                render_value_template(&ctx.templates, "b_output_svg", OutputFormat::LaTeX, s)
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
            CellOutput::Error { evalue, .. } => render_value_template(
                &ctx.templates,
                "b_output_error",
                OutputFormat::LaTeX,
                evalue,
            ),
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

                let mut args = TemplateContext::new();
                args.insert("cell_outputs", &ctx.metadata.cell_outputs);
                args.insert("source", &source);
                args.insert("id", &id);
                args.insert("tags", &tags);
                args.insert("outputs", &outputs.render(ctx)?);

                let output = ctx.templates.render("b_cell", OutputFormat::LaTeX, &args)?;
                Ok(output)
            }
            Block::List(idx, items) => {
                let inner: Result<String> = items.into_iter().map(|b| b.render(ctx)).collect();
                let inner = inner?;

                Ok(match idx {
                    None => render_value_template(
                        &ctx.templates,
                        "b_list_unordered.tera.tex",
                        OutputFormat::LaTeX,
                        inner,
                    )?,
                    Some(start) => {
                        let mut args = TemplateContext::new();
                        args.insert("start", &start);
                        args.insert("value", &inner);
                        ctx.templates
                            .render("b_list_ordered", OutputFormat::LaTeX, &args)?
                    }
                })
            }
            Block::ListItem(inner) => render_value_template(
                &ctx.templates,
                "b_list_item",
                OutputFormat::LaTeX,
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
    let mut args = ctx.extra_args.clone();

    let name = match shortcode {
        Shortcode::Inline(def) => {
            add_args(
                &mut args,
                def.id,
                def.num,
                &ctx.ids,
                &ctx.ids_map,
                render_params(def.parameters, ctx)?,
            );
            def.name
        }
        Shortcode::Block(def, body) => {
            let name = format!("shortcodes/tex/{}.tera.tex", def.name,);
            add_args(
                &mut args,
                def.id,
                def.num,
                &ctx.ids,
                &ctx.ids_map,
                render_params(def.parameters, ctx)?,
            );
            let body = body.render(ctx)?;
            args.insert("body", &body);
            def.name
        }
    };
    Ok(ctx.templates.render(&name, OutputFormat::LaTeX, &args)?)
}
