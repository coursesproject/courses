use crate::config::Format;
use anyhow::{anyhow, Context as AnyhowContext};
use rhai::{Dynamic, Engine, Scope};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{Cursor, Write};

use std::borrow::Borrow;
use std::io;
use std::path::PathBuf;

use tera::{Context, Filter, Function, Tera};

mod definition;

use crate::parsers::shortcodes::Parameter;
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

fn get_shortcode_tera_fn<'a>(
    temp: TemplateManager,
    id: String,
    format: Box<dyn Format>,
    type_: TemplateType,
) -> impl Function + 'a {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            let mut ctx = Context::new();
            args.iter().for_each(|(k, v)| {
                let s = &v.to_string();
                let len = s.len();
                ctx.insert(k, &s[1..len - 1]);
            });

            let mut buf = Cursor::new(Vec::new());
            match temp.render(&id, format.borrow(), type_.clone(), &ctx, &mut buf) {
                Ok(()) => Ok(Value::String(String::from_utf8(buf.into_inner()).unwrap())),
                Err(e) => {
                    let mut buf = Vec::new();
                    err_format(e, &mut buf)?;
                    Ok(Value::String(String::from_utf8(buf).unwrap()))
                }
            }
        },
    )
}

fn err_format(e: anyhow::Error, mut f: impl Write) -> io::Result<()> {
    write!(f, "Error {:?}", e)?;
    e.chain()
        .skip(1)
        .try_for_each(|cause| write!(f, " caused by: {}", cause))?;
    Ok(())
}

#[derive(Clone)]
pub struct TemplateManager {
    path: PathBuf,
    pub tera: Tera,
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

        let temp = TemplateManager {
            path: dir,
            tera,
            definitions,
        };

        let temp = temp.register_shortcode_fns()?;

        Ok(temp)
    }

    fn register_shortcode_fns(mut self) -> anyhow::Result<Self> {
        self.clone()
            .definitions
            .into_iter()
            .try_for_each(|(tp_name, def)| {
                let (_, id) = tp_name.split_once('_').unwrap();
                let type_ = &def.type_;
                for format in def.templates.keys() {
                    let format: Box<dyn Format> =
                        serde_json::from_str(&format!("{{\"{}\": {{}}}}", format))
                            .expect("problems!");

                    let f = get_shortcode_tera_fn(
                        self.clone(),
                        id.to_string(),
                        format.clone(),
                        type_.clone(),
                    );
                    let name = format!("shortcode_{format}_{id}");
                    self.tera.register_function(&name, f);
                }
                Ok::<(), anyhow::Error>(())
            })?;
        Ok(self)
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

        self.tera.render_to(&template_name, args, buf)?;
        Ok(())
    }

    pub fn validate_args_for_template(
        &self,
        id: &str,
        args: &[Parameter<String>],
    ) -> anyhow::Result<Vec<anyhow::Result<()>>> {
        let tp = self
            .get_template(id, TemplateType::Shortcode)
            .context(format!("Invalid shortcode identifier '{}'", id))?;
        tp.validate_args(args)
    }
}
