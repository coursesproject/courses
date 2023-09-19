use crate::templates::{Example, ShortcodeDefinition, TemplateSource, TemplateType};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Package {
    package: PackageMeta,
    dependencies: Vec<Dependency>,
    features: Vec<Feature>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageMeta {
    name: String,
    description: String,
    version: String,
    authors: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dependency {
    name: String,
    version: String,
    features: Vec<String>,
    #[serde(default = "default_root")]
    url_root: String,
}

fn default_root() -> String {
    "https://github.com".to_string()
}

// impl TryFrom<Dependency> for Package {
//     type Error = anyhow::Error;
//
//     fn try_from(value: Dependency) -> Result<Self, Self::Error> {
//         let url = format!("{}/{}", value.url_root, value.name);
//
//         Repository::submodule(&url)
//     }
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Feature {
    include_files: Vec<String>,
    include_template_prefixes: Vec<String>,
    dependency_features: Vec<String>,
}

impl Package {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensibleTemplateDefinition {
    pub extends: Option<String>,
    pub name: Option<String>,
    pub private: Option<bool>,
    pub value_template: Option<bool>,
    pub description: Option<String>,
    pub type_: Option<TemplateType>,
    pub script: Option<String>,
    pub shortcode: Option<ShortcodeDefinition>,
    pub templates: Option<HashMap<String, TemplateSource>>,
    pub examples: Option<Vec<Example>>,
}
