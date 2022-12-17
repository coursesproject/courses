use crate::project::{Project, ProjectConfig};
use cdoc::document::RawDocument;
use std::path::PathBuf;
use cdoc::processors::Preprocessor;

pub struct GeneratorContext {
    pub root: PathBuf,
    pub project: Project<RawDocument>,
    pub config: ProjectConfig,
    pub parser: Box<dyn Parser>>
}

pub trait Generator {
    fn generate(ctx: GeneratorContext) -> Self;
    fn rebuild_single(&self, doc: RawDocument);
}
