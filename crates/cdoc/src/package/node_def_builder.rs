use anyhow::anyhow;
use cdoc_base::node::definition::{NodeDef, Parameter, UsageExample};
use cdoc_base::template::TemplateSource;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Default, Clone, Debug)]
pub struct DefinitionFile(HashMap<String, NodeDefinitionDescriptor>);

#[derive(Deserialize, Default, Clone, Debug)]
pub struct NodeDefinitionDescriptor {
    description: Option<String>,
    parameters: Option<Vec<Parameter>>,
    children: Option<bool>,
    templates: Option<HashMap<String, TemplateSource>>,
    examples: Option<Vec<UsageExample>>,
}

impl NodeDefinitionDescriptor {
    pub fn combine(&mut self, other: Self) {
        if let Some(new_description) = other.description {
            self.description = Some(new_description);
        }
        if let Some(new_parameters) = other.parameters {
            self.parameters = Some(new_parameters);
        }
        if let Some(new_children) = other.children {
            self.children = Some(new_children);
        }

        if let Some(tp_other) = other.templates {
            if let Some(tp) = self.templates.as_mut() {
                for (key, value) in tp_other {
                    tp.insert(key, value);
                }
            } else {
                self.templates = Some(tp_other);
            }
        }

        if let Some(ex_other) = other.examples {
            if let Some(ex) = self.examples.as_mut() {
                ex.extend(ex_other);
            } else {
                self.examples = Some(ex_other);
            }
        }
    }

    pub fn load(&self, prefix: &str, base_path: &PathBuf) -> anyhow::Result<String> {
        match self
            .templates
            .as_ref()
            .unwrap()
            .get(prefix)
            .ok_or(anyhow!("Invalid prefix"))?
        {
            TemplateSource::String(s) => Ok(s.clone()),
            TemplateSource::File(p) => Ok(fs::read_to_string(base_path.join(p))?),
            TemplateSource::Derive(parent) => self.load(parent, base_path),
        }
    }

    pub fn resolve_templates(&mut self, base_path: &PathBuf) -> anyhow::Result<()> {
        if self.templates.is_some() {
            let mut new_templates = HashMap::default();

            for prefix in self.templates.as_ref().unwrap().keys() {
                let src = self.load(prefix, base_path)?;
                new_templates.insert(prefix.to_string(), TemplateSource::String(src));
            }

            self.templates = Some(new_templates);
        }

        Ok(())
    }

    pub fn build_definition(self, name: &str) -> anyhow::Result<NodeDef> {
        let description = self
            .description
            .ok_or_else(|| anyhow!("Missing description"))?;
        let parameters = self.parameters.unwrap_or_default();
        let children = self.children.unwrap_or_default();
        let templates = self
            .templates
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| match v {
                TemplateSource::String(s) => Ok((k, s)),
                _ => Err(anyhow!("A template has not been resolved")),
            })
            .collect::<anyhow::Result<HashMap<String, String>>>()?;
        let examples = self.examples.unwrap_or_default();

        Ok(NodeDef {
            name: name.to_string(),
            description,
            parameters,
            children,
            templates,
            examples,
        })
    }
}

impl DefinitionFile {
    pub fn combine(mut self, other: Self) -> Self {
        for (key, value) in other.0 {
            if let Some(first_value) = self.0.get_mut(&key) {
                first_value.combine(value);
            } else {
                self.0.insert(key, value);
            }
        }
        self
    }

    pub fn resolve_templates(mut self, base_path: &PathBuf) -> anyhow::Result<Self> {
        for v in self.0.values_mut() {
            v.resolve_templates(base_path)?;
        }
        Ok(self)
    }

    pub fn try_into_node_descriptions(self) -> anyhow::Result<Vec<NodeDef>> {
        self.0
            .into_iter()
            .map(|(k, v)| v.build_definition(&k))
            .collect()
    }
}
