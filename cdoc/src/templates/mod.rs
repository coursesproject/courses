use crate::config::OutputFormat;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};
use walkdir::WalkDir;

mod definition;
pub use definition::*;

#[derive(Debug, Clone)]
pub struct TemplateManager {
    tera: Tera,
    definitions: HashMap<String, TemplateDefinition>,
}

impl TemplateManager {
    pub fn new(definitions: HashMap<String, TemplateDefinition>) -> anyhow::Result<Self> {
        let defs = definitions.iter().flat_map(|(id, def)| {
            def.template_strings()
                .iter()
                .map(|(format, source)| {
                    let name = format!("{}_{}.{}", def.type_, def.name, format);
                    (name, source.clone())
                })
                .collect::<Vec<(String, String)>>()
        });
        let mut tera = Tera::new("")?;
        tera.add_raw_templates(defs)?;

        Ok(TemplateManager { tera, definitions })
    }

    pub fn get_template(&self, name: &str) -> anyhow::Result<TemplateDefinition> {
        let tp = self.definitions.get(name).ok_or(anyhow!("Invalid name"))?;
        Ok(tp.clone())
    }

    pub fn render(
        &self,
        name: &str,
        format: OutputFormat,
        args: &BTreeMap<&str, Value>,
    ) -> anyhow::Result<String> {
        let tp = self.get_template(name)?;
        tp.has_format(format)?;
        let type_ = &tp.type_;

        let template_name = format!("{type_}_{name}.{format}");

        let context = Context::from_serialize(args)?;
        Ok(self.tera.render(&template_name, &context)?)
    }

    pub fn validate_args_for_template(
        &self,
        name: &str,
        args: &BTreeMap<String, String>,
    ) -> Result<Vec<Result<(), ValidationError>>, anyhow::Error> {
        let def = self.definitions.get(name).ok_or(anyhow!("Invalid name"))?;
        Ok(def.validate_args(args)?)
    }
}
