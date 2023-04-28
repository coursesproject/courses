use crate::ast::Ast;
use crate::config::OutputFormat;
use anyhow::Result;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use tera::Tera;

use crate::document::Document;
use crate::notebook::NotebookMeta;
use crate::parsers::shortcodes::ShortCodeDef;
use crate::templates::TemplateManager;

pub mod html;
pub mod latex;
pub mod markdown;
pub mod notebook;

pub type RenderResult = String;

pub struct RenderContext {
    pub templates: TemplateManager,
    pub extra_args: BTreeMap<String, Value>,
    pub syntax_set: SyntaxSet,
    pub theme: Theme,
    pub notebook_output_meta: NotebookMeta,
}

#[typetag::serde(tag = "type")]
pub trait Renderer {
    fn render(
        &self,
        doc: &Document<Ast>,
        ctx: &RenderContext,
    ) -> anyhow::Result<Document<RenderResult>>;
}

pub struct RendererConfig {
    mapping: HashMap<String, Box<dyn Renderer>>,
}

impl RendererConfig {
    pub fn add_mapping(&mut self, extension: &str, parser: Box<dyn Renderer>) {
        self.mapping.insert(extension.to_string(), parser);
    }

    pub fn get_parser(&self, extension: &str) -> Option<&dyn Renderer> {
        self.mapping.get(extension).map(|b| b.deref())
    }
}

pub trait RenderElement<T> {
    fn render(self, ctx: &T) -> Result<String>;
}

impl<T, R: RenderElement<T>> RenderElement<T> for Vec<R> {
    fn render(self, ctx: &T) -> Result<String> {
        self.into_iter().map(|r| r.render(ctx)).collect()
    }
}

fn render_value_template(
    templates: &TemplateManager,
    name: &str,
    format: OutputFormat,
    value: String,
) -> Result<String> {
    let mut args = BTreeMap::new();
    args.insert("value", value.into());
    Ok(templates.render(name, format, &args)?)
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn add_args(
    ctx: &mut BTreeMap<&str, Value>,
    id: Option<String>,
    num: usize,
    ids: &HashMap<String, (usize, Vec<ShortCodeDef>)>,
    id_map: &HashMap<String, (usize, ShortCodeDef)>,
    arguments: HashMap<&str, String>,
) {
    if let Some(id) = id {
        ctx.insert("id", id.into());
    }
    ctx.insert("num", num.into());
    ctx.insert("ids", ids.into());
    ctx.insert("id_map", id_map.into());
    for (k, v) in arguments {
        ctx.insert(k, v.into());
    }
}
