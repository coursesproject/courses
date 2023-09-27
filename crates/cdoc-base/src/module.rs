use crate::node::NodeTypeDef;
use crate::package::TemplateDefinition;
use std::collections::HashMap;

pub struct Module {
    pub node_defs: HashMap<String, NodeTypeDef>,
    pub templates: HashMap<String, TemplateDefinition>,
}
