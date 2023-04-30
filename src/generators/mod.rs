use crate::pipeline::Mode;
use cdoc::document::Document;
use cdoc::renderers::RenderResult;
use cdoc::templates::TemplateManager;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tera::Tera;

use crate::project::config::ProjectConfig;
use crate::project::{ItemDescriptor, Project, ProjectResult};

pub mod html;
pub(crate) mod info;
pub mod latex;
pub mod markdown;
pub mod notebook;

#[derive(Clone)]
pub struct GeneratorContext<'a> {
    pub root: PathBuf,
    pub project: ProjectResult,
    pub templates: &'a TemplateManager,
    pub config: ProjectConfig,
    pub mode: Mode,
    pub build_dir: PathBuf,
}

pub trait Generator {
    fn generate(&self, ctx: &GeneratorContext) -> anyhow::Result<()>;
    fn generate_single(
        &self,
        content: Document<RenderResult>,
        doc_info: ItemDescriptor<()>,
        ctx: &GeneratorContext,
        // config: ProjectConfig,
        // build_dir: PathBuf,
    ) -> anyhow::Result<()>;
}
