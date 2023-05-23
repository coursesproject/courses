use crate::ast::Ast;
use crate::config::Format;
use anyhow::Result;

use std::collections::HashMap;
use std::io::Write;

use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use tera::Context;

use crate::document::Document;
use crate::notebook::NotebookMeta;
use crate::parsers::shortcodes::ShortCodeDef;
use crate::templates::TemplateManager;

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
