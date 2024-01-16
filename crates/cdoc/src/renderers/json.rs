use crate::renderers::extensions::RenderExtension;
use crate::renderers::{DocumentRenderer, RenderContext, RenderResult, RendererConfig};
use cdoc_base::document::Document;
use cdoc_base::node::Node;
use cowstr::CowStr;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct JsonRenderer;

#[typetag::serde(name = "json")]
impl RendererConfig for JsonRenderer {
    fn build(
        &self,
        extensions: Vec<Box<dyn RenderExtension>>,
    ) -> anyhow::Result<Box<dyn DocumentRenderer>> {
        Ok(Box::new(JsonRenderer))
    }
}

impl DocumentRenderer for JsonRenderer {
    fn render_doc(
        &mut self,
        doc: &Document<Vec<Node>>,
        ctx: &RenderContext,
    ) -> anyhow::Result<Document<RenderResult>> {
        let d = doc.clone();
        let new_content: CowStr = serde_json::to_string_pretty(&doc)?.into();
        Ok(d.map(|_| new_content.clone()))
    }
}
