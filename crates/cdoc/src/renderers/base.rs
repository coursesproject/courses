use crate::config::Format;
use crate::renderers::extensions::{RenderExtension, RenderExtensionContext};
use crate::renderers::{
    DocumentRenderer, RenderContext, RenderElement, RenderResult, RendererConfig,
};
use anyhow::{Context as Ctx, Result};
use cdoc_base::node::{Attribute, Compound, Node};

use cdoc_base::document::Document;
use linked_hash_map::LinkedHashMap;
use minijinja::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::{Cursor, Write};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ElementRendererConfig;

#[typetag::serde(name = "base")]
impl RendererConfig for ElementRendererConfig {
    fn build(
        &self,
        extensions: Vec<Box<dyn RenderExtension>>,
    ) -> Result<Box<dyn DocumentRenderer>> {
        Ok(Box::new(ElementRenderer::new(extensions)?))
    }
}

// #[derive(Debug)]
pub struct ElementRenderer {
    list_level: usize,
    current_list_idx: Vec<Option<u64>>, // todo: why aren't these ever read???
    counters: HashMap<String, usize>,
    extensions: HashMap<String, Box<dyn RenderExtension>>,
}

impl Display for ElementRenderer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ElementRenderer")
    }
}

// pub fn references_by_type(
//     refs: &mut LinkedHashMap<String, Reference>,
// ) -> HashMap<String, Vec<(String, Reference)>> {
//     let mut type_map = HashMap::new();
//     for (id, reference) in refs {
//         type_map
//             .entry(reference.type_id.to_string())
//             .or_insert(vec![])
//             .push((id.to_string(), reference.clone()));
//
//         reference.num = type_map.get(&reference.type_id).unwrap().len();
//     }
//     type_map
// }

impl ElementRenderer {
    pub fn new(extensions: Vec<Box<dyn RenderExtension>>) -> Result<Self> {
        let extensions = extensions
            .into_iter()
            .map(|e| (e.register_root_type(), e))
            .collect();

        Ok(Self {
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

impl DocumentRenderer for ElementRenderer {
    fn render_doc(
        &mut self,
        doc: &Document<Vec<Node>>,
        ctx: &RenderContext,
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

impl RenderElement<Node> for ElementRenderer {
    fn render(&mut self, elem: &Node, ctx: &RenderContext, mut buf: impl Write) -> Result<()> {
        match elem {
            Node::Plain(t) => buf.write_all(t.as_bytes())?,
            Node::Compound(n) => self.render(n, ctx, buf)?,
            _ => unreachable!(),
        };
        Ok(())
    }
}

impl RenderElement<Compound> for ElementRenderer {
    fn render(&mut self, elem: &Compound, ctx: &RenderContext, mut buf: impl Write) -> Result<()> {
        let mut args = ctx.extra_args.clone();

        if let Some(ext) = self.extensions.get_mut(&elem.type_id) {
            buf.write_all(
                ext.process(elem, &mut RenderExtensionContext::empty())?
                    .as_bytes(),
            )?;
            Ok(())
        } else {
            if elem
                .attributes
                .iter()
                .any(|(a, _)| a.as_ref().map(|v| v == "id").unwrap_or_default())
            {
                let num = self.fetch_and_inc_num(elem.type_id.clone());
                args.insert("num".to_string(), Value::from(num));
            }
            if let Some(id) = &elem.id {
                let num = self.fetch_and_inc_num(elem.type_id.clone());
                args.insert("num".to_string(), Value::from(num));
                args.insert("id".to_string(), Value::from(id.as_str()));
            }

            let params: LinkedHashMap<String, Attribute> = ctx
                .templates
                .resolve_params(&elem.type_id, elem.attributes.clone())?;
            let rendered = self
                .render_params(params, ctx)
                .with_context(|| format!("error rendering node {}", elem.type_id,))?;

            // let template_def = ctx
            //     .templates
            //     .get_template(&elem.type_id, TemplateType::Shortcode)?;
            // ctx.templates.validate_args_for_template(&elem.type_id, &rendered)?;

            add_args(&mut args, rendered)?;

            let body = self.render_inner(&elem.children, ctx)?;
            args.insert("body".to_string(), Value::from(body));

            args.insert("ctx".to_string(), Value::from_object(ctx.clone()));

            ctx.templates
                .render_template(ctx.format.template_prefix(), &elem.type_id, &args, buf)
        }
    }
}

fn add_args(args: &mut HashMap<String, Value>, arguments: Vec<RenderedParam>) -> Result<()> {
    for p in arguments {
        args.insert(p.key.to_string(), p.value);
    }
    Ok(())
}

pub struct RenderedParam {
    pub key: String,
    pub value: Value,
}

impl ElementRenderer {
    pub(crate) fn render_params(
        &mut self,
        parameters: LinkedHashMap<String, Attribute>,
        ctx: &RenderContext,
    ) -> Result<Vec<RenderedParam>> {
        parameters
            .into_iter()
            .map(|(k, attr)| {
                let value = match &attr {
                    Attribute::Flag => Value::from(k.clone()),
                    Attribute::Compound(inner) => Value::from(self.render_inner(inner, ctx)?),
                    Attribute::String(s) => Value::from(s.as_str()),
                    Attribute::Int(i) => Value::from(*i),
                    Attribute::Float(f) => Value::from(*f),
                    Attribute::Enum(s) => Value::from(s.as_str()),
                };

                Ok(RenderedParam { key: k, value })
            })
            .collect()
    }
}
