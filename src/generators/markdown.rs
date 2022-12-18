use crate::generators::{Generator, GeneratorContext};
use crate::project::config::ProjectConfig;
use crate::project::ProjectItem;
use anyhow::Error;
use cdoc::renderers::RenderResult;
use std::path::PathBuf;

pub struct MarkdownGenerator;

impl Generator for MarkdownGenerator {
    fn generate(&self, ctx: GeneratorContext) -> anyhow::Result<()> {
        // TODO: Generate notebook files AND copy resources!
        todo!()
    }

    fn generate_single(
        &self,
        content: RenderResult,
        doc_info: ProjectItem<()>,
        config: ProjectConfig,
        build_dir: PathBuf,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
