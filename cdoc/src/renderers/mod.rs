use crate::ast::Ast;
use crate::config::Format;
use anyhow::Result;

use std::collections::HashMap;

use std::sync::atomic::{AtomicUsize, Ordering};

use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;

use crate::document::Document;
use crate::notebook::NotebookMeta;
use crate::parsers::shortcodes::ShortCodeDef;
use crate::templates::{TemplateContext, TemplateManager, TemplateType};

pub mod generic;
pub mod notebook;

pub type RenderResult = String;

pub struct RenderContext<'a> {
    pub doc: &'a Document<Ast>,
    pub templates: &'a TemplateManager,
    pub extra_args: TemplateContext,
    pub syntax_set: SyntaxSet,
    pub theme: Theme,
    pub notebook_output_meta: NotebookMeta,
    pub format: &'a dyn Format,
    pub ids: HashMap<String, (usize, Vec<ShortCodeDef>)>,
    pub ids_map: HashMap<String, (usize, ShortCodeDef)>,
}

pub trait DocumentRenderer {
    fn render_doc(&mut self, ctx: &RenderContext) -> Result<Document<RenderResult>>;
}

pub trait RendererBuilder<'a> {
    type Renderer;
    fn build(self, doc: &'a Document<Ast>) -> Self::Renderer;
}

pub trait RenderElement<T> {
    fn render(&mut self, elem: &T, ctx: &RenderContext) -> Result<String>;
}

impl<T: RenderElement<R>, R> RenderElement<Vec<R>> for T {
    fn render(&mut self, elem: &Vec<R>, ctx: &RenderContext) -> Result<String> {
        elem.iter().map(|e| self.render(e, ctx)).collect()
    }
}

// impl<R: RenderElement> RenderElement for Vec<R> {
//     fn render(&mut self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<String> {
//         self.iter_mut().map(|r| r.render(doc, ctx)).collect()
//     }
// }

fn render_basic_template(name: &str, type_: TemplateType, ctx: &RenderContext) -> Result<String> {
    ctx.templates
        .render(name, ctx.format, type_, &TemplateContext::default())
}

fn render_value_template(
    name: &str,
    type_: TemplateType,
    value: &str,
    ctx: &RenderContext,
) -> Result<String> {
    let mut args = TemplateContext::default();
    args.insert("value", value);
    ctx.templates.render(name, ctx.format, type_, &args)
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn add_args(
    ctx: &mut TemplateContext,
    id: &Option<String>,
    num: usize,
    ids: &HashMap<String, (usize, Vec<ShortCodeDef>)>,
    id_map: &HashMap<String, (usize, ShortCodeDef)>,
    arguments: HashMap<String, String>,
) -> Result<()> {
    if let Some(id) = id {
        ctx.insert("id", &id);
    }
    ctx.insert("num", &num);
    ctx.insert("ids", &ids);
    ctx.insert("id_map", &id_map);
    for (k, v) in &arguments {
        ctx.insert(k, v);
    }
    Ok(())
}

fn render_image(url: &str, alt: &str, inner: &str, ctx: &RenderContext) -> Result<String> {
    let mut args = TemplateContext::default();
    args.insert("url", url);
    args.insert("alt", alt);
    args.insert("inner", inner);
    ctx.templates
        .render("image", ctx.format, TemplateType::Builtin, &args)
}

fn render_link(url: &str, alt: &str, inner: &str, ctx: &RenderContext) -> Result<String> {
    let mut args = TemplateContext::default();
    args.insert("url", url);
    args.insert("alt", alt);
    args.insert("inner", inner);
    ctx.templates
        .render("link", ctx.format, TemplateType::Builtin, &args)
}

fn render_math(
    display_mode: bool,
    trailing_space: bool,
    inner: &str,
    ctx: &RenderContext,
) -> Result<String> {
    let mut args = TemplateContext::default();
    args.insert("display_mode", &display_mode);
    args.insert("trailing_space", &trailing_space);
    args.insert("value", inner);
    ctx.templates
        .render("math", ctx.format, TemplateType::Builtin, &args)
}
