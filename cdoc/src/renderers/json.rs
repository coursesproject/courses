use crate::document::Document;
use crate::renderers::{DocumentRenderer, RenderContext, RenderResult};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct JsonRenderer;

#[typetag::serde(name = "json")]
impl DocumentRenderer for JsonRenderer {
    fn render_doc(&mut self, ctx: &RenderContext) -> anyhow::Result<Document<RenderResult>> {
        ctx.doc
            .clone()
            .try_map(|a| Ok(serde_json::to_string_pretty(&a.0)?))
    }
}
