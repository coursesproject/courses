use crate::package::Dist;
use anyhow::anyhow;
use cdoc_base::node::definition::NodeDef;
use cdoc_base::node::Attribute;
use minijinja::Environment;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone)]
pub struct NewTemplateManager {
    environments: HashMap<String, Environment<'static>>,
    nodes: HashMap<String, NodeDef>,
}

impl NewTemplateManager {
    pub fn new(module_build: Dist) -> anyhow::Result<Self> {
        let mut environments = HashMap::new();
        let nodes: HashMap<String, NodeDef> = module_build
            .nodes
            .into_iter()
            .map(|n| (n.name.clone(), n))
            .collect();

        for node in nodes.values() {
            for (prefix, src) in &node.templates {
                let env = environments
                    .entry(prefix.to_string())
                    .or_insert_with(|| Environment::new());
                env.add_template_owned(node.name.clone(), src.clone())?;
            }
        }

        for (name, layout) in module_build.layouts {
            for (prefix, src) in layout.0.iter() {
                let env = environments
                    .entry(prefix.to_string())
                    .or_insert_with(|| Environment::new());
                // println!("layout {}", name);
                env.add_template_owned(format!("layout_{name}"), src.clone())?;
            }
        }

        Ok(NewTemplateManager {
            environments,
            nodes,
        })
    }

    pub fn render<S: Serialize>(
        &self,
        prefix: &str,
        node: &str,
        ctx: S,
        buf: impl Write,
    ) -> anyhow::Result<()> {
        self.environments
            .get(prefix)
            .ok_or_else(|| anyhow!("Invalid prefix"))?
            .get_template(node)?
            .render_to_write(ctx, buf)?;
        Ok(())
    }

    pub fn resolve_params(
        &self,
        node: &str,
        params: Vec<(Option<String>, Attribute)>,
    ) -> anyhow::Result<BTreeMap<String, Attribute>> {
        let def = self
            .nodes
            .get(node)
            .ok_or(anyhow!("Node of type '{node}' does not exist"))?;
        let mut out = BTreeMap::new();

        for (i, (name, val)) in params.iter().enumerate() {
            if let Some(n) = name {
                out.insert(n.clone(), val.clone());
            } else {
                let key = def.parameters.get(i).ok_or(anyhow!(
                    "Invalid parameter position {i} for node of type {}",
                    def.name
                ))?;
                out.insert(key.name.clone(), val.clone());
            }
        }

        Ok(out)
    }

    pub fn render_layout<S: Serialize>(
        &self,
        prefix: &str,
        name: &str,
        ctx: S,
        buf: impl Write,
    ) -> anyhow::Result<()> {
        self.environments
            .get(prefix)
            .ok_or_else(|| anyhow!("Invalid prefix"))?
            .get_template(&format!("layout_{name}"))?
            .render_to_write(ctx, buf)?;
        Ok(())
    }

    pub fn reload(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}
