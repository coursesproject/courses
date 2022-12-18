use cdoc::renderers::RenderResult;
use std::path::PathBuf;

use crate::project::config::ProjectConfig;
use crate::project::{Project, ProjectItem};

pub mod config;
pub mod html;
pub mod markdown;
pub mod notebook;

#[derive(Clone)]
pub struct GeneratorContext {
    pub root: PathBuf,
    pub project: Project<Option<RenderResult>>,
    pub config: ProjectConfig,
    pub build_dir: PathBuf,
}

pub trait Generator {
    fn generate(&self, ctx: GeneratorContext) -> anyhow::Result<()>;
    fn generate_single(
        &self,
        content: RenderResult,
        doc_info: ProjectItem<()>,
        config: ProjectConfig,
        build_dir: PathBuf,
    ) -> anyhow::Result<()>;
}
