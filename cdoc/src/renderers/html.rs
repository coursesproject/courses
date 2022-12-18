use pulldown_cmark::html;
use serde::{Deserialize, Serialize};

use crate::document::EventDocument;
use crate::renderers::{RenderResult, Renderer};

#[derive(Serialize, Deserialize)]
pub struct HtmlRenderer;

#[typetag::serde(name = "renderer_config")]
impl Renderer for HtmlRenderer {
    fn render(&self, doc: &EventDocument) -> RenderResult {
        let iter = doc.to_events();
        let mut output = String::new();
        html::push_html(&mut output, iter);
        RenderResult {
            content: output,
            metadata: doc.metadata.clone(),
            variables: doc.variables.clone(),
        }
    }
}
