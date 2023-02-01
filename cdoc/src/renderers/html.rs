use crate::document::{Document, EventContent};
use pulldown_cmark::html;
use serde::{Deserialize, Serialize};
use crate::ast::{AEvent, Ast};

use crate::renderers::{RenderResult, Renderer};

#[derive(Serialize, Deserialize)]
pub struct HtmlRenderer {
    pub(crate) interactive_cells: bool,
}

#[typetag::serde(name = "renderer_config")]
impl Renderer for HtmlRenderer {
    fn render(&self, doc: &Document<EventContent>) -> Document<RenderResult> {
        let iter = doc.to_events();

        let ast: Ast = iter.collect();

        let iter: Vec<AEvent> = ast.into_iter().collect();
        let iter = iter.iter().map(|aevent| aevent.into());

        let mut output = String::new();
        html::push_html(&mut output, iter);
        Document {
            content: output,
            metadata: doc.metadata.clone(),
            variables: doc.variables.clone(),
        }
    }
}
