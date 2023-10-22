use crate::config::Format;
use crate::parser::ParserSettings;
use crate::renderers::extensions::{RenderExtension, RenderExtensionContext};
use crate::renderers::parameter_resolution::ParameterResolution;
use crate::renderers::{
    DocumentRenderer, RenderContext, RenderElement, RenderResult, RendererConfig,
};
use crate::templates::{TemplateDefinition, TemplateManager, TemplateType};
use anyhow::{Context as Ctx, Result};
// use cdoc_base::node::into_rhai::build_types;
use cdoc_base::node::visitor::NodeVisitor;
use cdoc_base::node::{Attribute, Compound, Node};

use crate::renderers::references::Reference;
use cdoc_base::document::Document;
use linked_hash_map::LinkedHashMap;
use rhai::{Array, Dynamic, Engine, ImmutableString, Scope, AST};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{BufWriter, Cursor, Write};
use tera::Context as TeraContext;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ElementRendererConfig {
    list_level: usize,
    current_list_idx: Vec<Option<u64>>,
    counters: HashMap<String, usize>,
}


pub struct ElementRenderer<'a> {
    engine: Engine,
    ast: AST,
    scope: Scope<'a>,
    fns: HashSet<String>,
    list_level: usize,
    current_list_idx: Vec<Option<u64>>,
    counters: HashMap<String, usize>,
    extensions: HashMap<String, Box<dyn RenderExtension>>,
}

pub fn references_by_type(
    refs: &mut LinkedHashMap<String, Reference>,
) -> HashMap<String, Vec<(String, Reference)>> {
    let mut type_map = HashMap::new();
    for (id, reference) in refs {
        type_map
            .entry(reference.obj_type.to_string())
            .or_insert(vec![])
            .push((id.to_string(), reference.clone()));

        reference.num = type_map.get(&reference.obj_type).unwrap().len();
    }
    type_map
}

impl ElementRenderer<'_> {
    pub fn new(src: &str, extensions: Vec<Box<dyn RenderExtension>>) -> Result<Self> {
        let mut engine = Engine::new();
        // build_types(&mut engine);
        let ast = engine.compile(src)?;
        let mut scope = Scope::new();
        engine.run_ast_with_scope(&mut scope, &ast)?;

        let fns = ast.iter_functions().map(|m| m.name.to_string()).collect();

        let extensions = extensions
            .into_iter()
            .map(|e| (e.register_root_type(), e))
            .collect();

        Ok(Self {
            engine,
            scope,
            ast,
            fns,
            list_level: 0,
            current_list_idx: vec![],
            counters: Default::default(),
            extensions,
        })
    }

    fn fetch_and_inc_num(&mut self, typ: String) -> usize {
        let num = self.counters.entry(typ).or_insert(0);
        *num += 1;

        *num
    }
}

impl DocumentRenderer for ElementRenderer<'_> {
    fn render_doc(
        &mut self,
        doc: &Document<Vec<Node>>,
        ctx: &mut RenderContext,
    ) -> Result<Document<RenderResult>> {
        let buf = Vec::new();
        let mut cursor = Cursor::new(buf);
        self.render(&doc.content, ctx, &mut cursor)?;

        let content = String::from_utf8(cursor.get_ref().clone())?.into();
        Ok(Document {
            content,
            meta: doc.meta.clone(),
            code_outputs: doc.code_outputs.clone(),
        })
    }
}

impl RenderElement<Node> for ElementRenderer<'_> {
    fn render(&mut self, elem: &Node, ctx: &mut RenderContext, mut buf: impl Write) -> Result<()> {
        Ok(match elem {
            Node::Plain(t) => buf.write_all(t.as_bytes())?,
            Node::Compound(n) => self.render(n, ctx, buf)?,
            _ => unreachable!(),
        })
    }
}

impl RenderElement<Compound> for ElementRenderer<'_> {
    fn render(
        &mut self,
        elem: &Compound,
        ctx: &mut RenderContext,
        mut buf: impl Write,
    ) -> Result<()> {
        let mut args = ctx.extra_args.clone();

        if let Some(ext) = self.extensions.get_mut(&elem.type_id) {
            buf.write_all(
                ext.process(elem, &mut RenderExtensionContext::empty())?
                    .as_bytes(),
            )?;
            Ok(())
        } else {
            if let Some(_) = elem.attributes.get("label") {
                let num = self.fetch_and_inc_num(elem.type_id.clone());
                args.insert("num", &num);
            }

            let rendered = self
                .render_params(elem.attributes.clone(), ctx)
                .with_context(|| format!("error rendering node {}", elem.type_id,))?;

            // let template_def = ctx
            //     .templates
            //     .get_template(&elem.type_id, TemplateType::Shortcode)?;
            // ctx.templates.validate_args_for_template(&elem.type_id, &rendered)?;

            add_args(&mut args, rendered)?;

            let body = self.render_inner(&elem.children, ctx)?;
            args.insert("body", &body);

            ctx.templates.render(
                &elem.type_id,
                ctx.format.template_prefix(),
                TemplateType::Shortcode,
                &args,
                buf,
            )
        }
    }
}

fn add_args(args: &mut TeraContext, arguments: Vec<RenderedParam>) -> Result<()> {
    for p in arguments {
        args.insert(p.key.as_str(), &p.value);
    }
    Ok(())
}

pub struct RenderedParam {
    pub key: String,
    pub value: Value,
}

impl ElementRenderer<'_> {
    pub(crate) fn render_params(
        &mut self,
        parameters: BTreeMap<String, Attribute>,
        ctx: &mut RenderContext,
    ) -> Result<Vec<RenderedParam>> {
        parameters
            .into_iter()
            .map(|(k, attr)| {
                let value = match &attr {
                    Attribute::Flag => Value::Null,
                    Attribute::Compound(inner) => Value::String(self.render_inner(inner, ctx)?),
                    Attribute::String(s) => Value::String(s.to_string()),
                    Attribute::Int(i) => Value::Number((*i).into()),
                    Attribute::Float(f) => Value::Number(Number::from_f64(*f).unwrap()),
                    Attribute::Enum(s) => Value::String(s.to_string()),
                };

                Ok(RenderedParam { key: k, value })
            })
            .collect()
    }
}
