use std::fs;

use cdoc::document::Document;
use cdoc::renderers::RenderResult;

use crate::generators::{Generator, GeneratorContext};
use crate::project::ItemDescriptor;

pub struct InfoGenerator;

impl Generator for InfoGenerator {
    fn generate(&self, ctx: &GeneratorContext) -> anyhow::Result<()> {
        let output = serde_yaml::to_string(&ctx.project)?;
        let path = ctx.build_dir.join("config.yml");
        fs::write(path, output)?;
        Ok(())
    }

    fn generate_single(
        &self,
        _content: Document<RenderResult>,
        _doc_info: ItemDescriptor<()>,
        ctx: &GeneratorContext,
    ) -> anyhow::Result<()> {
        self.generate(ctx)
    }
}
