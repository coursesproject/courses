use crate::node::NodeTypeDef;

use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub struct Module {
    pub(crate) node_defs: HashMap<String, NodeTypeDef>,
    // pub(crate) templates: HashMap<String, TemplateDefinition>,
}

impl Module {
    pub fn get_node_def(&self, type_id: &str) -> Result<&NodeTypeDef> {
        self.node_defs
            .get(type_id)
            .ok_or_else(|| anyhow!("Not a valid node type"))
    }
}
