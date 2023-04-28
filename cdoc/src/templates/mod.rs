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
pub struct TemplateContext {
    pub map: BTreeMap<String, Value>,
}

impl TemplateContext {
    pub fn new() -> Self {
        TemplateContext {
            map: BTreeMap::new(),
        }
    }

    pub fn to_tera_context(&self) -> anyhow::Result<Context> {
        Ok(Context::from_serialize(self.map.clone())?)
    }

    pub fn insert<V: Serialize + ?Sized>(&mut self, key: &str, val: &V) {
        self.map.insert(
            key.to_string(),
            serde_json::to_value(val).expect("Invalid value"),
        );
    }
}

#[derive(Debug, Clone)]
pub struct TemplateManager {
    tera: Tera,
    definitions: HashMap<String, TemplateDefinition>,
}

impl TemplateManager {
    pub fn from_path(path: PathBuf) -> anyhow::Result<Self> {
        TemplateManager::new(load_template_definitions(path.clone())?, path)
    }

    fn new(definitions: HashMap<String, TemplateDefinition>, dir: PathBuf) -> anyhow::Result<Self> {
        let defs = definitions.iter().flat_map(|(id, def)| {
            def.template_strings(dir.join("sources"))
                .iter()
                .map(|(format, source)| {
                    let name = format!("{}_{}.{}", def.type_, def.name, format);
                    (name, source.clone())
                })
                .collect::<Vec<(String, String)>>()
        });
        let mut tera = Tera::new(&format!("{}/sources/**", dir.to_str().unwrap()))?;
        tera.add_raw_templates(defs)?;

        Ok(TemplateManager { tera, definitions })
    }

    pub fn get_template(&self, name: &str) -> anyhow::Result<TemplateDefinition> {
        let tp = self.definitions.get(name).ok_or(anyhow!(
            "Template definition with name {} doesn't exist.",
            name
        ))?;
        Ok(tp.clone())
    }

    pub fn render(
        &self,
        name: &str,
        format: OutputFormat,
        args: &TemplateContext,
    ) -> anyhow::Result<String> {
        let tp = self.get_template(name)?;
        tp.has_format(format)?;
        let type_ = &tp.type_;

        let template_name = format!("{type_}_{name}.{format}");

        let context = args.to_tera_context()?;
        Ok(self.tera.render(&template_name, &context)?)
    }

    pub fn validate_args_for_template(
        &self,
        name: &str,
        args: &TemplateContext,
    ) -> Result<Vec<Result<(), ValidationError>>, anyhow::Error> {
        let def = self.definitions.get(name).ok_or(anyhow!("Invalid name"))?;
        Ok(def.validate_args(args)?)
    }
}
