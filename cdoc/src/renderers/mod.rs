use crate::ast::{Ast, Inline};
use anyhow::Result;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use syntect::highlighting::Theme;
use syntect::parsing::{SyntaxReference, SyntaxSet};
use tera::Tera;

use crate::document::Document;
use crate::renderers::html::ToHtmlContext;

pub mod html;
pub mod latex;
pub mod markdown;
pub mod notebook;

pub type RenderResult = String;

pub struct RenderContext {
    pub tera: Tera,
    pub syntax_set: SyntaxSet,
    pub theme: Theme,
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

fn render_value_template(tera: &Tera, template: &str, value: String) -> Result<String> {
    let mut context = tera::Context::new();
    context.insert("value", &value);
    let output = tera.render(template, &context)?;
    Ok(output)
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
