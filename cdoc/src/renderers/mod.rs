pub mod html;
pub mod markdown;
pub mod notebook;

use crate::document::EventDocument;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;

#[typetag::serde(tag = "type")]
pub trait Renderer {
    fn render(&self, doc: &EventDocument) -> String;
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
