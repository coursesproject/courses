use crate::renderers::extensions::RenderExtension;
use crate::renderers::{DocumentRenderer, RenderContext, RenderResult};
use cdoc_parser::document::Document;
use cowstr::CowStr;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct JsonRenderer;

// #[typetag::serde(name = "json")]
impl DocumentRenderer for JsonRenderer {
    fn render_doc(
        &mut self,
        ctx: &mut RenderContext,
        _extensions: Vec<Box<dyn RenderExtension>>,
    ) -> anyhow::Result<Document<RenderResult>> {
        let d = ctx.doc.clone();
        let new_content: CowStr = serde_json::to_string_pretty(&ctx.doc)?.into();
        Ok(d.map(|_| new_content.clone()))
    }
}
