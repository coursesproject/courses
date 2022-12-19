use std::path::PathBuf;

use cdoc::renderers::RenderResult;

use crate::generators::{Generator, GeneratorContext};
use crate::project::config::ProjectConfig;
use crate::project::ProjectItem;

pub struct ConfigGenerator;

impl Generator for ConfigGenerator {
    fn generate(&self, _ctx: GeneratorContext) -> anyhow::Result<()> {
        // TODO: Generate notebook files AND copy resources!
        todo!()

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
        _content: RenderResult,
        _doc_info: ProjectItem<()>,
        _config: ProjectConfig,
        _build_dir: PathBuf,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
