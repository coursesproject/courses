pub mod structure;

use anyhow::Result;
use cdoc_base::node::{Attribute, Compound};

use dyn_clone::DynClone;
use linked_hash_map::LinkedHashMap;
use std::fmt::Debug;

pub struct RenderExtensionContext {
    meta: LinkedHashMap<String, Attribute>,
}

impl RenderExtensionContext {
    pub fn empty() -> Self {
        Self {
            meta: LinkedHashMap::new(),
        }
    }
}

pub trait RenderExtension: DynClone {
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
dyn_clone::clone_trait_object!(RenderExtension);
