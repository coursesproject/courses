use crate::ast::Ast;
use crate::config::Format;
use anyhow::Result;

use std::collections::HashMap;
use std::io::Write;

use std::sync::atomic::{AtomicUsize, Ordering};

use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use tera::Context;

use crate::document::Document;
use crate::notebook::NotebookMeta;
use crate::parsers::shortcodes::{Parameter, ShortCodeDef};
use crate::templates::{TemplateDefinition, TemplateManager, TemplateType};

pub mod generic;
pub mod notebook;

pub type RenderResult = String;

pub struct RenderContext<'a> {
    pub doc: &'a Document<Ast>,
    pub templates: &'a TemplateManager,
    pub extra_args: Context,
    pub syntax_set: SyntaxSet,
    pub theme: Theme,
    pub notebook_output_meta: &'a NotebookMeta,
    pub format: &'a dyn Format,
    pub ids: &'a HashMap<String, (usize, Vec<ShortCodeDef>)>,
    pub ids_map: &'a HashMap<String, (usize, ShortCodeDef)>,
}

pub trait DocumentRenderer {
    fn render_doc(&mut self, ctx: &RenderContext) -> Result<Document<RenderResult>>;
}

pub trait RendererBuilder<'a> {
    type Renderer;
    fn build(self, doc: &'a Document<Ast>) -> Self::Renderer;
}

pub trait RenderElement<T> {
    fn render(&mut self, elem: &T, ctx: &RenderContext, buf: impl Write) -> Result<()>;

    fn render_inner(&mut self, elem: &T, ctx: &RenderContext) -> Result<String> {
        let mut buf = Vec::new();
        self.render(elem, ctx, &mut buf)?;
        Ok(String::from_utf8(buf)?)
    }
}

impl<T: RenderElement<R>, R> RenderElement<Vec<R>> for T {
    fn render(&mut self, elem: &Vec<R>, ctx: &RenderContext, mut buf: impl Write) -> Result<()> {
        elem.iter().try_for_each(|e| self.render(e, ctx, &mut buf))
    }
}

// impl<R: RenderElement> RenderElement for Vec<R> {
//     fn render(&mut self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<String> {
//         self.iter_mut().map(|r| r.render(doc, ctx)).collect()
//     }
// }

fn render_basic_template(
    name: &str,
    type_: TemplateType,
    ctx: &RenderContext,
    buf: impl Write,
) -> Result<()> {
    ctx.templates
        .render(name, ctx.format, type_, &Context::default(), buf)
}

fn render_value_template(
    name: &str,
    type_: TemplateType,
    value: &str,
    ctx: &RenderContext,
    buf: impl Write,
) -> Result<()> {
    let mut args = Context::default();
    args.insert("value", value);
    ctx.templates.render(name, ctx.format, type_, &args, buf)
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn add_args(
    def: &TemplateDefinition,
    args: &mut Context,
    id: &Option<String>,
    num: usize,
    ids: &HashMap<String, (usize, Vec<ShortCodeDef>)>,
    id_map: &HashMap<String, (usize, ShortCodeDef)>,
    arguments: Vec<Parameter<String>>,
) -> Result<()> {
    if let Some(id) = id {
        args.insert("id", &id);
    }
    args.insert("num", &num);
    args.insert("ids", &ids);
    args.insert("id_map", &id_map);
    for (i, p) in arguments.into_iter().enumerate() {
        match p {
            Parameter::Positional { value } => args.insert(
                def.shortcode.as_ref().unwrap().parameters[i].name.clone(),
                value.inner(),
            ),
            Parameter::Keyword { name, value } => args.insert(name, value.inner()),
        }
    }
    Ok(())
}

fn render_image(
    url: &str,
    alt: &str,
    inner: &str,
    ctx: &RenderContext,
    buf: impl Write,
) -> Result<()> {
    let mut args = Context::default();
    args.insert("url", url);
    args.insert("alt", alt);
    args.insert("inner", inner);
    ctx.templates
        .render("image", ctx.format, TemplateType::Builtin, &args, buf)
}

fn render_link(
    url: &str,
    alt: &str,
    inner: &str,
    ctx: &RenderContext,
    buf: impl Write,
) -> Result<()> {
    let mut args = Context::default();
    args.insert("url", url);
    args.insert("alt", alt);
    args.insert("inner", inner);
    ctx.templates
        .render("link", ctx.format, TemplateType::Builtin, &args, buf)
}

fn render_math(
    display_mode: bool,
    trailing_space: bool,
    inner: &str,
    ctx: &RenderContext,
    buf: impl Write,
) -> Result<()> {
    let mut args = Context::default();
    args.insert("display_mode", &display_mode);
    args.insert("trailing_space", &trailing_space);
    args.insert("value", inner);
    ctx.templates
        .render("math", ctx.format, TemplateType::Builtin, &args, buf)
}
