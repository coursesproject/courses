use std::path::PathBuf;

use cdoc::renderers::RenderResult;

use crate::generators::{Generator, GeneratorContext};
use crate::project::config::ProjectConfig;
use crate::project::ProjectItem;

pub struct MarkdownGenerator;

impl Generator for MarkdownGenerator {
    fn generate(&self, _ctx: GeneratorContext) -> anyhow::Result<()> {
        // TODO: Generate notebook files AND copy resources!
        todo!()
    }

    fn generate_single(
        &self,
        _content: RenderResult,
        _doc_info: ProjectItem<()>,
        _config: ProjectConfig,
        _build_dir: PathBuf,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
