use crate::config::{Format, OutputFormat};
use anyhow::{anyhow, Context as AnyhowContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tera::{Context, Filter, Tera};
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
    path: PathBuf,
    tera: Tera,
    pub definitions: HashMap<String, TemplateDefinition>,
}

impl TemplateManager {
    pub fn from_path(path: PathBuf) -> anyhow::Result<Self> {
        TemplateManager::new(load_template_definitions(path.clone())?, path)
    }

    fn new(definitions: HashMap<String, TemplateDefinition>, dir: PathBuf) -> anyhow::Result<Self> {
        let defs = get_templates_from_definitions(&definitions, dir.clone());
        let mut tera = Tera::new(&format!("{}/sources/**.html", dir.to_str().unwrap()))?;
        tera.add_raw_templates(defs)?;
        Ok(TemplateManager {
            path: dir,
            tera,
            definitions,
        })
    }

    pub fn reload(&mut self) -> anyhow::Result<()> {
        let defs = load_template_definitions(self.path.clone())?;
        let tps = get_templates_from_definitions(&defs, self.path.clone());
        self.tera.full_reload()?;
        self.tera.add_raw_templates(tps)?;
        self.definitions = defs;
        Ok(())
    }

    pub fn register_filter<F: Filter + 'static>(&mut self, name: &str, filter: F) {
        self.tera.register_filter(name, filter)
    }

    pub fn get_template(
        &self,
        id: &str,
        type_: TemplateType,
    ) -> anyhow::Result<TemplateDefinition> {
        let tp = self
            .definitions
            .get(&format!("{type_}_{id}"))
            .ok_or(anyhow!(
                "Template definition with id '{}' and type '{}' doesn't exist.",
                id,
                type_
            ))?;
        Ok(tp.clone())
    }

    pub fn render(
        &self,
        id: &str,
        format: &dyn Format,
        type_: TemplateType,
        args: &TemplateContext,
    ) -> anyhow::Result<String> {
        let tp = self.get_template(id, type_)?;
        let format_str = format.template_name();
        tp.has_format(format_str).context(format!(
            "template with id '{id}' does not support format '{format_str}"
        ))?;
        let type_ = &tp.type_;

        let template_name = format!("{type_}_{id}.{format_str}");

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
