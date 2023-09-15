use crate::renderers::{DocumentRenderer, RenderContext, RenderResult};
use cdoc_parser::document::Document;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct JsonRenderer;

#[typetag::serde(name = "json")]
impl DocumentRenderer for JsonRenderer {
    fn render_doc(&mut self, ctx: &RenderContext) -> anyhow::Result<Document<RenderResult>> {
        let d = ctx.doc.clone();
        let new_content = serde_json::to_string_pretty(&ctx.doc)?;
        Ok(d.map(|_| new_content.clone()))
    }
}
