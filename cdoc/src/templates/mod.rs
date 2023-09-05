use anyhow::{anyhow, Context as AnyhowContext};
use rhai::{Dynamic, Engine, Scope};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{Cursor, Write};

use rhai::serde::{from_dynamic, to_dynamic};
use std::io;
use std::path::PathBuf;

use tera::{Context, Filter, Function, Tera};

mod definition;
mod precompiled;

use crate::parsers::shortcodes::{Argument, ShortCodeCall};
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
    template_prefix: String,
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
            match temp.render(&id, &template_prefix, type_.clone(), &ctx, &mut buf) {
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

/// Provides a common Api for the three layout types and output formats.
#[derive(Clone)]
pub struct TemplateManager {
    path: PathBuf,
    pub tera: Tera,
    pub definitions: HashMap<String, TemplateDefinition>,
    filter_path: PathBuf,
}

impl TemplateManager {
    /// Create new template manager from template path. Reads the template files.
    pub fn from_path(
        template_path: PathBuf,
        filter_path: PathBuf,
        create_filters: bool,
    ) -> anyhow::Result<Self> {
        TemplateManager::new(
            load_template_definitions(template_path.clone())?,
            template_path,
            filter_path,
            create_filters,
        )
    }

    fn new(
        definitions: HashMap<String, TemplateDefinition>,
        dir: PathBuf,
        filter_path: PathBuf,
        create_filters: bool,
    ) -> anyhow::Result<Self> {
        let defs = get_templates_from_definitions(&definitions, dir.clone());
        let mut tera = Tera::new(&format!("{}/sources/**.html", dir.to_str().unwrap()))?;
        let filters = get_filters_from_files(filter_path.clone())?;

        filters.into_iter().for_each(|(name, source)| {
            tera.register_filter(&name, create_rhai_filter(source));
        });

        tera.add_raw_templates(defs)?;

        let temp = TemplateManager {
            path: dir,
            tera,
            definitions,
            filter_path,
        };

        Ok(if create_filters {
            temp.register_shortcode_fns()?
        } else {
            temp
        })
    }

    #[allow(unused)]
    fn combine(mut self, other: TemplateManager) -> anyhow::Result<TemplateManager> {
        self.tera.extend(&other.tera)?;
        self.definitions.extend(other.definitions);

        Ok(self)
    }

    fn register_shortcode_fns(mut self) -> anyhow::Result<Self> {
        self.clone()
            .definitions
            .into_iter()
            .try_for_each(|(tp_name, def)| {
                let (_, id) = tp_name.split_once('_').unwrap();
                let type_ = &def.type_;
                for template_prefix in def.templates.keys() {
                    let f = get_shortcode_tera_fn(
                        self.clone(),
                        id.to_string(),
                        template_prefix.clone(),
                        type_.clone(),
                    );
                    let name = format!("shortcode_{template_prefix}_{id}");
                    self.tera.register_function(&name, f);
                }
                Ok::<(), anyhow::Error>(())
            })?;
        Ok(self)
    }

    /// Reload all files and definitions
    pub fn reload(&mut self) -> anyhow::Result<()> {
        let defs = load_template_definitions(self.path.clone())?;
        let tps = get_templates_from_definitions(&defs, self.path.clone());
        self.tera.full_reload()?;
        self.tera.add_raw_templates(tps)?;
        let filters = get_filters_from_files(self.filter_path.clone())?;

        filters.into_iter().for_each(|(name, source)| {
            self.tera.register_filter(&name, create_rhai_filter(source));
        });

        self.definitions = defs;
        Ok(())
    }

    /// Register Tera filter
    pub fn register_filter<F: Filter + 'static>(&mut self, name: &str, filter: F) {
        self.tera.register_filter(name, filter)
    }

    /// Fetch a [TemplateDefinition] by specifying its id and type.
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

    /// Render a template to a specified format
    ///
    /// # Arguments
    ///
    /// * `id` - The template identifier (the name of the definition file)
    /// * `template_prefix` - The template format key (which output format to use)
    /// * `type_` - The kind of template to render (builtin/layout/shortcode). Ensures that
    ///     different types can have templates with the same id.
    /// * `args` - Template arguments contained in a Tera context.
    /// * `buf` - Buffer to write the output to.
    pub fn render(
        &self,
        id: &str,
        template_prefix: &str,
        type_: TemplateType,
        args: &Context,
        buf: impl Write,
    ) -> anyhow::Result<()> {
        let tp = self.get_template(id, type_)?;
        let format_str = template_prefix;
        let format = tp.get_format(format_str).context(format!(
            "template with id '{id}' does not support format '{format_str}"
        ))?;
        let args = if let Some(script) = &tp.script {
            let engine = Engine::new();
            let mut scope = Scope::new();
            scope.push_dynamic("args", to_dynamic(args.clone().into_json())?);
            engine
                .run_with_scope(&mut scope, script)
                .context("running script")?;

            let args = scope.get_value::<Dynamic>("args").expect("args missing");
            let args_value = from_dynamic(&args)?;
            Context::from_value(args_value).expect("invalid type")
        } else {
            args.clone()
        };
        match format {
            TemplateSource::Precompiled(tp, fm) => {
                tp.render(fm, &args, buf)?;
            }
            TemplateSource::Derive(from) => {
                let format = tp.get_format(from).context(format!(
                    "template with id '{id}' does not support format '{format_str}"
                ))?;
                if let TemplateSource::Precompiled(tp, fm) = format {
                    tp.render(fm, &args, buf)?;
                } else {
                    let type_ = &tp.type_;

                    let template_name = format!("{type_}_{id}.{format_str}");

                    self.tera.render_to(&template_name, &args, buf)?;
                }
            }
            _ => {
                let type_ = &tp.type_;

                let template_name = format!("{type_}_{id}.{format_str}");

                self.tera.render_to(&template_name, &args, buf)?;
            }
        }

        Ok(())
    }

    /// Performs argument validation for shortcodes.
    pub fn validate_args_for_template(
        &self,
        id: &str,
        args: &[Argument<String>],
    ) -> anyhow::Result<Vec<anyhow::Result<()>>> {
        let tp = self
            .get_template(id, TemplateType::Shortcode)
            .context(format!("Invalid shortcode identifier '{}'", id))?;
        tp.validate_args(args)
    }

    pub fn shortcode_call_resolve_positionals(
        &self,
        call: ShortCodeCall,
    ) -> anyhow::Result<ShortCodeCall> {
        let tp = self.get_template(&call.name, TemplateType::Shortcode)?;
        let params = tp.shortcode.unwrap().parameters;
        let args = call
            .arguments
            .into_iter()
            .enumerate()
            .map(|(i, a)| match a {
                Argument::Keyword { name, value } => Argument::Keyword { name, value },
                Argument::Positional { value } => Argument::Keyword {
                    name: params.get(i).unwrap().name.clone(),
                    value,
                },
            })
            .collect();
        Ok(ShortCodeCall {
            name: call.name,
            id: call.id,
            arguments: args,
        })
    }
}
