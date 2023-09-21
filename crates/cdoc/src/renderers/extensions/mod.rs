pub mod structure;

use crate::renderers::generic::GenericRenderer;
use crate::renderers::{RenderContext, RenderElement};
use cdoc_parser::ast::{Block, Inline};
use dyn_clone::DynClone;
use std::fmt::Debug;

pub trait RenderExtension {
    fn name(&self) -> String;
    fn process(&mut self, ctx: &mut RenderContext, renderer: GenericRenderer)
        -> anyhow::Result<()>;
}

#[typetag::serde]
pub trait RenderExtensionConfig: Debug + Send + Sync + DynClone {
    fn build(&self) -> anyhow::Result<Box<dyn RenderExtension>>;
}

pub fn build_extensions(
    extensions: &Vec<Box<dyn RenderExtensionConfig>>,
) -> anyhow::Result<Vec<Box<dyn RenderExtension>>> {
    extensions.iter().map(|e| e.build()).collect()
}

dyn_clone::clone_trait_object!(RenderExtensionConfig);
