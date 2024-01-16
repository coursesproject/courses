use std::fmt::{Display, Formatter};

use crate::renderers::base::ElementRenderer;
use crate::renderers::{RenderContext, RenderElement};
use crate::templates::new::NewTemplateManager;
use cdoc_base::node::visitor::NodeVisitor;
use cdoc_base::node::{Attribute, Compound};
use linked_hash_map::LinkedHashMap;
use minijinja::value::Object;
use minijinja::{Error, ErrorKind, State, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub id: String,
    pub type_id: String,
    pub attr: LinkedHashMap<String, Value>, // pub attr: LinkedHashMap<CowStr, CowStr>,
    pub num: usize,
}

pub struct ReferenceVisitor<'a> {
    pub(crate) references: LinkedHashMap<String, Vec<Reference>>,
    templates: &'a NewTemplateManager,
}

impl<'a> ReferenceVisitor<'a> {
    pub fn new(templates: &'a NewTemplateManager) -> Self {
        ReferenceVisitor {
            references: Default::default(),
            templates,
        }
    }

    pub fn reference_map(&'a self) -> LinkedHashMap<String, Reference> {
        let mut output = LinkedHashMap::new();
        for group in self.references.values() {
            for reference in group.iter() {
                output.insert(reference.id.clone(), reference.clone());
            }
        }
        output
    }
}

#[derive(Debug)]
struct AttributeRenderer(Attribute);

impl Display for AttributeRenderer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AttributeRenderer")
    }
}

impl Object for AttributeRenderer {
    fn call_method(&self, _state: &State, name: &str, args: &[Value]) -> Result<Value, Error> {
        match name {
            "render" => match &self.0 {
                Attribute::Int(v) => Ok(Value::from(*v)),
                Attribute::Float(v) => Ok(Value::from(*v)),
                Attribute::String(v) => Ok(Value::from(v.as_str())),
                Attribute::Enum(v) => Ok(Value::from(v.as_str())),
                Attribute::Compound(n) => {
                    let a = args.get(0).ok_or(Error::new(
                        ErrorKind::MissingArgument,
                        "RenderContext missing",
                    ))?;
                    let ctx = a.downcast_object_ref::<RenderContext>().ok_or(Error::new(
                        ErrorKind::CannotUnpack,
                        "Invalid RenderContext type",
                    ))?;

                    let mut renderer = ElementRenderer::new(vec![])
                        .map_err(|e| Error::new(ErrorKind::InvalidOperation, e.to_string()))?;
                    let res = renderer
                        .render_inner(n, ctx)
                        .map_err(|e| Error::new(ErrorKind::InvalidOperation, e.to_string()))?;
                    Ok(Value::from(res))
                }
                Attribute::Flag => Ok(Value::from("flag")),
            },
            _ => Err(Error::new(ErrorKind::UnknownMethod, "Unknown method")),
        }
    }
}

impl NodeVisitor for ReferenceVisitor<'_> {
    fn visit_compound(&mut self, node: &mut Compound) -> anyhow::Result<()> {
        if let Some(id) = &node.id {
            let attrs = self
                .templates
                .resolve_params(&node.type_id, node.attributes.clone())?;
            let attrs = attrs
                .iter()
                .map(|(k, attr)| {
                    (
                        k.to_string(),
                        Value::from_object(AttributeRenderer(attr.clone())),
                    )
                })
                .collect();

            let list = self.references.entry(node.type_id.clone()).or_default();
            let reference = Reference {
                type_id: node.type_id.clone(),
                id: id.to_string(),
                attr: attrs,
                num: list.len(),
            };
            list.push(reference);
        }

        self.walk_compound(node)
    }
}

//TODO: Implement
//
// impl AstVisitor for ReferenceVisitor {
//     fn visit_code_block(&mut self, block: &mut CodeBlock) -> anyhow::Result<()> {
//         if let Some(label) = &block.label {
//             self.references.insert(
//                 label.to_string(),
//                 Reference {
//                     obj_type: "code".to_string(),
//                     attr: Default::default(), // TODO: Attrs
//                     num: 0,
//                 },
//             );
//         }
//         Ok(())
//     }
//
//     fn visit_math(&mut self, math: &mut Math) -> anyhow::Result<()> {
//         if let Some(label) = &math.label {
//             self.references.insert(
//                 label.to_string(),
//                 Reference {
//                     obj_type: "equation".to_string(),
//                     attr: Default::default(),
//                     num: 0,
//                 },
//             );
//         }
//         Ok(())
//     }
//
//     fn visit_command(&mut self, cmd: &mut Command) -> anyhow::Result<()> {
//         let params = cmd
//             .parameters
//             .iter()
//             .filter_map(|p| {
//                 p.key.as_ref().and_then(|k| match &p.value {
//                     Value::String(s) => Some((k.clone(), s.clone())),
//                     _ => None,
//                 })
//             })
//             .collect();
//         if let Some(id) = &cmd.label {
//             self.references.insert(
//                 id.to_string(),
//                 Reference {
//                     obj_type: cmd.function.to_string(),
//                     attr: params,
//                     num: 0,
//                 },
//             );
//         }
//         if let Some(body) = &mut cmd.body {
//             self.walk_vec_block(body)?;
//         }
//         Ok(())
//     }
//
//     // TODO: Math block
// }
