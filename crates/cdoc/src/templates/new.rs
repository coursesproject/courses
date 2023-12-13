use crate::package::Dist;
use anyhow::anyhow;
use cdoc_base::node::definition::NodeDef;
use minijinja::Environment;
use serde::Serialize;
use std::collections::HashMap;
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
