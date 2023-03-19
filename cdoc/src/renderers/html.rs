use crate::ast::{Ast, Block, Inline};
use crate::document::{Document, DocumentMetadata};
use crate::notebook::{CellOutput, OutputValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use tera::Tera;

use crate::renderers::{RenderContext, RenderResult, Renderer};

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
            tera: ctx.tera.clone(),
        };

        Ok(Document {
            content: doc.content.0.clone().to_html(&ctx)?,
            metadata: doc.metadata.clone(),
            variables: doc.variables.clone(),
        })
    }
}

pub struct ToHtmlContext {
    pub metadata: DocumentMetadata,
    pub tera: Tera,
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
            Inline::Emphasis(inner) | Inline::Strong(inner) | Inline::Strikethrough(inner) => {
                inner.to_html(ctx)
            }
            Inline::Code(s) => Ok(s),
            Inline::SoftBreak => Ok(String::default()),
            Inline::HardBreak => Ok(String::default()),
            Inline::Rule => Ok("<hr>".to_string()),
            Inline::Image(_tp, url, title) => Ok(format!(r#"<img src="{url}">{title}</img>"#)),
            Inline::Link(_tp, url, title) => Ok(format!(r#"<a href="{url}">{title}</a>"#)),
            Inline::Html(s) => Ok(s),
        }
    }
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn render_value_template(ctx: &ToHtmlContext, template: &str, value: String) -> Result<String> {
    let mut context = tera::Context::new();
    context.insert("value", &value);
    let output = ctx.tera.render(template, &context)?;
    Ok(output)
}

impl ToHtml for OutputValue {
    fn to_html(self, ctx: &ToHtmlContext) -> Result<String> {
        match self {
            OutputValue::Plain(s) => render_value_template(ctx, "html/output_text.tera.html", s),
            OutputValue::Image(s) => render_value_template(ctx, "html/img.tera.html", s),
            OutputValue::Svg(s) => render_value_template(ctx, "html/svg.tera.html", s),
            OutputValue::Json(_) => Ok("".to_string()),
            OutputValue::Html(_) => Ok("".to_string()),
            OutputValue::Javascript(_) => Ok("".to_string()),
        }
    }
}

impl ToHtml for CellOutput {
    fn to_html(self, ctx: &ToHtmlContext) -> Result<String> {
        match self {
            CellOutput::Stream { text, .. } => Ok(text),
            CellOutput::Data { data, .. } => data.into_iter().map(|v| v.to_html(ctx)).collect(),
            CellOutput::Error { evalue, .. } => {
                render_value_template(ctx, "html/output_error.tera.html", evalue)
            }
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
                source, outputs, ..
            } => {
                let id = get_id();

                let mut context = tera::Context::new();
                context.insert("interactive", &ctx.metadata.interactive.unwrap_or_default());
                context.insert("cell_outputs", &ctx.metadata.cell_outputs);
                context.insert("editable", &ctx.metadata.editable.unwrap_or_default());
                context.insert("source", &source);
                context.insert("id", &id);
                context.insert("outputs", &outputs.to_html(ctx)?);

                let output = ctx.tera.render("html/cell.tera.html", &context)?;
                Ok(output)
            }
            Block::List(idx, items) => {
                let inner: Result<String> = items.into_iter().map(|b| b.to_html(ctx)).collect();
                let inner = inner?;
                Ok(match idx {
                    None => format!("<ul>{inner}</ul>"),
                    Some(start) => format!(r#"<ol start="{start}">{inner}</ol>"#),
                })
            }
            Block::ListItem(inner) => Ok(format!("<li>{}</li>", inner.to_html(ctx)?)),
        }
    }
}
