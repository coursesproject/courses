use crate::config::Format;
use anyhow::{anyhow, Context as AnyhowContext};
use rhai::{Dynamic, Engine, Scope};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;

use std::path::PathBuf;

use tera::{Context, Filter, Tera};

mod definition;

pub use definition::*;

fn create_rhai_filter(source: String) -> impl Filter {
    Box::new(
        move |val: &Value, args: &HashMap<String, Value>| -> tera::Result<Value> {
            let eng = Engine::new();
            let mut scope = Scope::new();
            scope.push("val", val.clone());
            scope.push("args", args.clone());
            let res: Dynamic = eng.eval_with_scope(&mut scope, &source).unwrap();

            Ok(serde_json::to_value(res).unwrap())
        },
    )
}

#[derive(Clone)]
pub struct TemplateManager {
    path: PathBuf,
    tera: Tera,
    pub definitions: HashMap<String, TemplateDefinition>,
}

impl TemplateManager {
    pub fn from_path(template_path: PathBuf, filter_path: PathBuf) -> anyhow::Result<Self> {
        TemplateManager::new(
            load_template_definitions(template_path.clone())?,
            template_path,
            filter_path,
        )
    }

    fn new(
        definitions: HashMap<String, TemplateDefinition>,
        dir: PathBuf,
        filter_path: PathBuf,
    ) -> anyhow::Result<Self> {
        let defs = get_templates_from_definitions(&definitions, dir.clone());
        let filters = get_filters_from_files(filter_path)?;
        let mut tera = Tera::new(&format!("{}/sources/**.html", dir.to_str().unwrap()))?;

        filters.into_iter().for_each(|(name, source)| {
            tera.register_filter(&name, create_rhai_filter(source));
        });

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
        args: &Context,
        buf: impl Write,
    ) -> anyhow::Result<()> {
        let tp = self.get_template(id, type_)?;
        let format_str = format.template_name();
        tp.has_format(format_str).context(format!(
            "template with id '{id}' does not support format '{format_str}"
        ))?;
        let type_ = &tp.type_;

        let template_name = format!("{type_}_{id}.{format_str}");

        self.tera.render_to(&template_name, &args, buf)?;
        Ok(())
    }

    pub fn validate_args_for_template(
        &self,
        name: &str,
        args: &Context,
    ) -> Result<Vec<Result<(), ValidationError>>, anyhow::Error> {
        let def = self.definitions.get(name).ok_or(anyhow!("Invalid name"))?;
        def.validate_args(args)
    }
}
