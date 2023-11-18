use crate::template::TemplateSource;
use anyhow::anyhow;
use minijinja::Environment;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub struct NodeDescription {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) parameters: Vec<Parameter>,
    pub(crate) children: bool,
    pub(crate) templates: HashMap<String, TemplateSource>,
}

#[derive(Deserialize)]
pub struct Parameter {
    name: String,
    type_: String,
    optional: bool,
}

#[derive(Deserialize)]
pub struct ParameterType {}

impl NodeDescription {
    pub fn load(&self, prefix: &str, base_path: PathBuf) -> anyhow::Result<String> {
        match self
            .templates
            .get(prefix)
            .ok_or(anyhow!("Invalid prefix"))?
        {
            TemplateSource::String(s) => Ok(s.clone()),
            TemplateSource::File(p) => Ok(fs::read_to_string(base_path.join(p))?),
            TemplateSource::Derive(parent) => self.load(parent, base_path),
        }
    }
}

pub struct TemplateManager {
    environments: HashMap<String, Environment<'static>>,
    nodes: HashMap<String, NodeDescription>,
}

impl TemplateManager {
    pub fn new(nodes: Vec<NodeDescription>, base_path: PathBuf) -> anyhow::Result<Self> {
        let mut environments = HashMap::new();
        let nodes: HashMap<String, NodeDescription> =
            nodes.into_iter().map(|n| (n.name.clone(), n)).collect();

        for node in nodes.values() {
            for prefix in node.templates.keys() {
                let src_str = node.load(&prefix, base_path.clone())?;
                let env = environments
                    .entry(prefix.to_string())
                    .or_insert_with(|| Environment::new());
                env.add_template_owned(node.name.clone(), src_str)?;
            }
        }

        Ok(TemplateManager {
            environments,
            nodes,
            gf,
        })
    }
}
