use crate::node::Node;
use anyhow::Result;
use linked_hash_map::LinkedHashMap;
use serde_json::Value;

pub struct ExtensionContext {
    meta: LinkedHashMap<String, Value>,
}
pub trait Extension {
    fn run(&mut self, nodes: &mut [Node], ctx: &mut ExtensionContext) -> Result<()>;
}
