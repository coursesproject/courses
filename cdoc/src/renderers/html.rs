use crate::ast::{Ast, Block, Inline, Shortcode};
use crate::config::OutputFormat;
use crate::document::{Document, DocumentMetadata};
use crate::notebook::{CellOutput, OutputValue, StreamType};
use crate::parsers::shortcodes::ShortCodeDef;
use crate::renderers;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use tera::Tera;

use crate::renderers::{
    add_args, render_image, render_link, render_math, RenderContext, RenderResult, Renderer,
};
use crate::templates::{TemplateContext, TemplateManager};

#[derive(Serialize, Deserialize)]
pub struct HtmlRenderer {
    pub(crate) interactive_cells: bool,
}

#[typetag::serde(name = "renderer_config")]
impl Renderer for HtmlRenderer {
    fn render(&self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<Document<RenderResult>> {
        // let doc = doc.to_events();
        // let dd = doc.to_events();
        //
        // let mut output = String::new();
        // html::push_html(&mut output, dd);
        let ctx = ToHtmlContext {
            metadata: doc.metadata.clone(),
            ids: doc.ids.clone(),
            ids_map: doc.id_map.clone(),
            templates: ctx.templates.clone(),
            extra_args: ctx.extra_args.clone(),
            syntax_set: ctx.syntax_set.clone(),
            theme: ctx.theme.clone(),
        };

        Ok(Document {
            content: doc.content.0.clone().to_html(&ctx)?,
            metadata: doc.metadata.clone(),
            variables: doc.variables.clone(),
            ids: doc.ids.clone(),
            id_map: doc.id_map.clone(),
        })
    }
}

pub struct ToHtmlContext {
    pub metadata: DocumentMetadata,
    pub ids: HashMap<String, (usize, Vec<ShortCodeDef>)>,
    pub ids_map: HashMap<String, (usize, ShortCodeDef)>,
    pub templates: TemplateManager,
    pub extra_args: TemplateContext,
    pub syntax_set: SyntaxSet,
    pub theme: Theme,
}

pub trait ToHtml {
    fn to_html(self, ctx: &ToHtmlContext) -> Result<String>;
}

impl ToHtml for Vec<Inline> {
    fn to_html(self, ctx: &ToHtmlContext) -> Result<String> {
        self.into_iter().map(|i| i.to_html(ctx)).collect()
    }
}

impl ToHtml for Vec<Block> {
    fn to_html(self, ctx: &ToHtmlContext) -> Result<String> {
        self.into_iter().map(|b| b.to_html(ctx)).collect()
    }
}

impl ToHtml for Inline {
    fn to_html(self, ctx: &ToHtmlContext) -> Result<String> {
        match self {
            Inline::Text(s) => Ok(s),
            Inline::Emphasis(inner) => Ok(format!("<em>{}</em>", inner.to_html(ctx)?)),
            Inline::Strong(inner) => Ok(format!("<strong>{}</strong>", inner.to_html(ctx)?)),
            Inline::Strikethrough(inner) => Ok(format!("<s>{}</s>", inner.to_html(ctx)?)),
            Inline::Code(s) => Ok(format!("<code>{}</code>", s)),
            Inline::SoftBreak => Ok("<br>".to_string()),
            Inline::HardBreak => Ok("<br>".to_string()),
            Inline::Rule => Ok("<hr>".to_string()),
            Inline::Image(_tp, url, alt, inner) => {
                let inner_s = inner.to_html(ctx)?;
                render_image(&url, &alt, &inner_s, &ctx.templates, OutputFormat::Html)
            }
            Inline::Link(_tp, url, alt, inner) => {
                let inner_s = inner.to_html(ctx)?;
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
            Inline::Shortcode(s) => {
                Ok(render_shortcode_template(ctx, s).unwrap_or_else(|e| e.to_string()))
            }
        }
    }
}

impl ToHtml for OutputValue {
    fn to_html(self, ctx: &ToHtmlContext) -> Result<String> {
        match self {
            OutputValue::Plain(s) => renderers::render_value_template(
                &ctx.templates,
                "output_text",
                OutputFormat::Html,
                s.join(""),
            ),
            OutputValue::Image(s) => renderers::render_value_template(
                &ctx.templates,
                "output_img",
                OutputFormat::Html,
                s,
            ),
            OutputValue::Svg(s) => renderers::render_value_template(
                &ctx.templates,
                "output_svg",
                OutputFormat::Html,
                s,
            ),
            OutputValue::Json(s) => Ok(serde_json::to_string(&s)?),
            OutputValue::Html(s) => Ok(s),
            OutputValue::Javascript(_) => Ok("".to_string()),
        }
    }
}

impl ToHtml for CellOutput {
    fn to_html(self, ctx: &ToHtmlContext) -> Result<String> {
        match self {
            CellOutput::Stream { text, name } => match name {
                StreamType::StdOut => renderers::render_value_template(
                    &ctx.templates,
                    "b_output_text",
                    OutputFormat::Html,
                    text,
                ),
                StreamType::StdErr => renderers::render_value_template(
                    &ctx.templates,
                    "b_output_error",
                    OutputFormat::Html,
                    text,
                ),
            },
            CellOutput::Data { data, .. } => data.into_iter().map(|v| v.to_html(ctx)).collect(),
            CellOutput::Error { evalue, .. } => renderers::render_value_template(
                &ctx.templates,
                "b_output_error",
                OutputFormat::Html,
                evalue,
            ),
        }
    }
}

impl ToHtml for Vec<CellOutput> {
    fn to_html(self, ctx: &ToHtmlContext) -> Result<String> {
        self.into_iter().map(|o| o.to_html(ctx)).collect()
    }
}

impl ToHtml for Block {
    fn to_html(self, ctx: &ToHtmlContext) -> Result<String> {
        match self {
            Block::Heading { lvl, inner, .. } => {
                Ok(format!("<{lvl}>{}</{lvl}>", inner.to_html(ctx)?))
            }
            Block::Plain(inner) => inner.to_html(ctx),
            Block::Paragraph(inner) | Block::BlockQuote(inner) => inner.to_html(ctx),
            Block::CodeBlock {
                source,
                outputs,
                tags,
                ..
            } => {
                let id = renderers::get_id();

                let highlighted = syntect::html::highlighted_html_for_string(
                    &source,
                    &ctx.syntax_set,
                    ctx.syntax_set.find_syntax_by_extension("py").unwrap(),
                    &ctx.theme,
                )?;

                let mut args = TemplateContext::new();
                args.insert("interactive", &ctx.metadata.interactive);
                args.insert("cell_outputs", &ctx.metadata.cell_outputs);
                args.insert("editable", &ctx.metadata.editable);
                args.insert("source", &source);
                args.insert("highlighted", &highlighted);
                args.insert("id", &id);
                args.insert("tags", &tags);
                args.insert("outputs", &outputs.to_html(ctx)?);

                Ok(ctx.templates.render("b_cell", OutputFormat::Html, &args)?)
            }
            Block::List(idx, items) => {
                let inner: Result<String> = items.into_iter().map(|b| b.to_html(ctx)).collect();
                let inner = inner?;

                Ok(match idx {
                    None => renderers::render_value_template(
                        &ctx.templates,
                        "b_list_unordered",
                        OutputFormat::Html,
                        inner,
                    )?,
                    Some(start) => {
                        let mut args = TemplateContext::new();
                        args.insert("start", &start);
                        args.insert("value", &inner);
                        ctx.templates
                            .render("b_list_ordered", OutputFormat::Html, &args)?
                    }
                })
            }
            Block::ListItem(inner) => renderers::render_value_template(
                &ctx.templates,
                "list_item",
                OutputFormat::Html,
                inner.to_html(ctx)?,
            ),
        }
    }
}

fn render_params(
    parameters: HashMap<String, Vec<Block>>,
    ctx: &ToHtmlContext,
) -> Result<HashMap<String, String>> {
    parameters
        .into_iter()
        .map(|(k, v)| Ok((k, v.to_html(ctx)?)))
        .collect()
}

fn render_shortcode_template(ctx: &ToHtmlContext, shortcode: Shortcode) -> Result<String> {
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
            add_args(
                &mut args,
                def.id,
                def.num,
                &ctx.ids,
                &ctx.ids_map,
                render_params(def.parameters, ctx)?,
            );
            let body = body.to_html(ctx)?;
            args.insert("body", &body);
            def.name
        }
    };
    Ok(ctx.templates.render(&name, OutputFormat::Html, &args)?)
}
