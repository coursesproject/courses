use std::collections::HashMap;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::document::{DocumentMetadata, DocumentVariables, EventDocument};

pub mod html;
pub mod markdown;
pub mod notebook;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderResult {
    pub content: String,
    pub metadata: DocumentMetadata,
    pub variables: DocumentVariables,
}

#[typetag::serde(tag = "type")]
pub trait Renderer {
    fn render(&self, doc: &EventDocument) -> RenderResult;
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
