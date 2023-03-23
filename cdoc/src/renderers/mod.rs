use crate::ast::Ast;
use std::collections::HashMap;
use std::ops::Deref;
use syntect::highlighting::Theme;
use syntect::parsing::{SyntaxReference, SyntaxSet};
use tera::Tera;

use crate::document::Document;

pub mod html;
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
