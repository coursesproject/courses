use crate::renderers::extensions::RenderExtension;
use crate::renderers::{DocumentRenderer, RenderContext, RenderResult};
use cdoc_base::document::Document;
use cdoc_base::node::Node;
use cowstr::CowStr;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct JsonRenderer;

// #[typetag::serde(name = "json")]
impl DocumentRenderer for JsonRenderer {
    fn render_doc(
        &mut self,
        doc: &Document<Vec<Node>>,
        ctx: &mut RenderContext,
    ) -> anyhow::Result<Document<RenderResult>> {
        let d = doc.clone();
        let new_content: CowStr = serde_json::to_string_pretty(&doc)?.into();
        Ok(d.map(|_| new_content.clone()))
    }
}
