pub mod cell_outputs;
pub mod structure;

use crate::renderers::RenderContext;

use crate::renderers::newrenderer::ElementRenderer;
use dyn_clone::DynClone;
use std::fmt::Debug;

pub trait RenderExtension {
    fn name(&self) -> String;
    fn process(
        &mut self,
        ctx: &mut RenderContext,
        renderer: &ElementRenderer,
    ) -> anyhow::Result<()>;
}

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
