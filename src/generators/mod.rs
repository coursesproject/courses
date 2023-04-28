use cdoc::document::Document;
use cdoc::renderers::RenderResult;
use cdoc::templates::TemplateManager;
use std::path::PathBuf;
use tera::Tera;

use crate::project::config::ProjectConfig;
use crate::project::{ItemDescriptor, Project};

pub mod html;
pub(crate) mod info;
pub mod latex;
pub mod markdown;
pub mod notebook;

#[derive(Clone)]
pub struct GeneratorContext {
    pub root: PathBuf,
    pub project: Project<Option<Document<RenderResult>>>,
    pub templates: TemplateManager,
    pub config: ProjectConfig,
    pub build_dir: PathBuf,
}

pub trait Generator {
    fn generate(&self, ctx: GeneratorContext) -> anyhow::Result<()>;
    fn generate_single(
        &self,
        content: Document<RenderResult>,
        doc_info: ItemDescriptor<()>,
        ctx: GeneratorContext,
        // config: ProjectConfig,
        // build_dir: PathBuf,
    ) -> anyhow::Result<()>;
}
