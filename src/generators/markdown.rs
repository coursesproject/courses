use cdoc::document::Document;
use std::path::PathBuf;

use cdoc::renderers::RenderResult;

use crate::generators::{Generator, GeneratorContext};
use crate::project::config::ProjectConfig;
use crate::project::ItemDescriptor;

pub struct MarkdownGenerator;

impl Generator for MarkdownGenerator {
    fn generate(&self, _ctx: GeneratorContext) -> anyhow::Result<()> {
        // TODO: Generate notebook files AND copy resources!
        todo!()
    }

    fn generate_single(
        &self,
        _content: Document<RenderResult>,
        _doc_info: ItemDescriptor<()>,
        _config: ProjectConfig,
        _build_dir: PathBuf,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
