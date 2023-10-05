pub mod cell_outputs;
pub mod structure;

use crate::renderers::Document;

use crate::renderers::newrenderer::ElementRenderer;
use anyhow::Result;
use cdoc_base::node::{Compound, Node};
use cdoc_parser::ast::Value;
use dyn_clone::DynClone;
use linked_hash_map::LinkedHashMap;
use std::fmt::Debug;

pub struct RenderExtensionContext {
    meta: LinkedHashMap<String, Value>,
}

impl RenderExtensionContext {
    pub fn empty() -> Self {
        Self {
            meta: LinkedHashMap::new(),
        }
    }
}

pub trait RenderExtension {
    fn register_root_type(&self) -> String;
    fn process(&mut self, element: &Compound, ctx: &mut RenderExtensionContext) -> Result<String>;
}

// pub trait RenderExtension {
//     fn name(&self) -> String;
//     fn process(
//         &mut self,
//         ctx: &mut RenderContext,
//         renderer: &ElementRenderer,
//     ) -> anyhow::Result<()>;
// }

#[typetag::serde]
pub trait RenderExtensionConfig: Debug + Send + Sync + DynClone {
    fn build(&self) -> anyhow::Result<Box<dyn RenderExtension>>;
}

pub fn build_extensions(
    extensions: &[Box<dyn RenderExtensionConfig>],
) -> anyhow::Result<Vec<Box<dyn RenderExtension>>> {
    extensions.iter().map(|e| e.build()).collect()
}

dyn_clone::clone_trait_object!(RenderExtensionConfig);
