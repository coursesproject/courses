use crate::package::node_description::{NodeDescription, Parameter};
use crate::template::TemplateSource;
use anyhow::anyhow;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct DefinitionFile(HashMap<String, NodeDescriptionElement>);

#[derive(Deserialize)]
pub struct NodeDescriptionElement {
    description: Option<String>,
    parameters: Option<Vec<Parameter>>,
    children: Option<bool>,
    templates: Option<HashMap<String, TemplateSource>>,
}

impl NodeDescriptionElement {
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
    }

    pub fn try_into_description(self, name: &str) -> anyhow::Result<NodeDescription> {
        let description = self
            .description
            .ok_or_else(|| anyhow!("Missing description"))?;
        let parameters = self.parameters.unwrap_or_default();
        let children = self.children.unwrap_or_default();
        let templates = self.templates.ok_or_else(|| anyhow!("Missing templates"))?;

        Ok(NodeDescription {
            name: name.to_string(),
            description,
            parameters,
            children,
            templates,
        })
    }
}

impl DefinitionFile {
    pub fn combine(&mut self, other: Self) {
        for (key, value) in other.0 {
            if let Some(first_value) = self.0.get_mut(&key) {
                first_value.combine(value)
            }
        }
    }

    pub fn try_into_node_descriptions(self) -> anyhow::Result<Vec<NodeDescription>> {
        self.0
            .into_iter()
            .map(|(k, v)| v.try_into_description(&k))
            .collect()
    }
}
