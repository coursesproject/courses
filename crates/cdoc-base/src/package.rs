use crate::module::Module;
use crate::node::NodeTypeDef;
use crate::template::TemplateSource;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Package {
    name: String,
    version: Version,
    #[serde(flatten)]
    provides: PackageProvides,
    dependencies: HashMap<String, DependencySpec>,
}

#[derive(Deserialize)]
pub struct DependencySpec {
    version: VersionReq,
}

#[derive(Deserialize)]
pub struct PackageProvides {
    node_defs: Vec<NodeTypeDef>,
    templates: Vec<TemplateDefinition>,
}

#[derive(Deserialize)]
pub struct TemplateDefinition {
    type_id: String,
    sources: HashMap<String, TemplateSource>,
}

impl Package {
    pub fn resolve(self) -> Module {
        Module {
            node_defs: self
                .provides
                .node_defs
                .into_iter()
                .map(|d| (d.type_id(), d))
                .collect(),
            templates: self
                .provides
                .templates
                .into_iter()
                .map(|t| (t.type_id(), t))
                .collect(),
        }
    }
}
