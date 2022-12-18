use crate::generators::{Generator, GeneratorContext};
use crate::project::config::ProjectConfig;
use crate::project::ProjectItem;
use anyhow::Error;
use cdoc::renderers::RenderResult;
use std::path::PathBuf;

pub struct ConfigGenerator;

impl Generator for ConfigGenerator {
    fn generate(&self, ctx: GeneratorContext) -> anyhow::Result<()> {
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
        content: RenderResult,
        doc_info: ProjectItem<()>,
        config: ProjectConfig,
        build_dir: PathBuf,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
