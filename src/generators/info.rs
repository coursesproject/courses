use cdoc::document::Document;
use std::fs;
use std::path::PathBuf;

use cdoc::renderers::RenderResult;

use crate::generators::{Generator, GeneratorContext};
use crate::project::config::ProjectConfig;
use crate::project::ItemDescriptor;

pub struct InfoGenerator;

impl Generator for InfoGenerator {
    fn generate(&self, _ctx: GeneratorContext) -> anyhow::Result<()> {
        // TODO: Generate notebook files AND copy resources!

        let output = serde_yaml::to_string(&_ctx.project)?;
        let path = _ctx.build_dir.join("config.yml");
        fs::write(path, output)?;
        Ok(())
        //
        //
        // serde_yaml::to_writer(
        //     &File::create(proj.project_path.as_path().join("build").join("config.yml")).unwrap(),
        //     &cf,
        // )
        //     .unwrap();
    }

    fn generate_single(
        &self,
        _content: Document<RenderResult>,
        _doc_info: ItemDescriptor<()>,
        _ctx: GeneratorContext,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
