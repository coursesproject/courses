use cdoc::document::Document;
use cdoc::parser::ParserSettings;
use cdoc::parsers::split::parse_code_string;
use cdoc::parsers::split_types::{CodeTaskDefinition, Output};
use cdoc::renderers::RenderResult;
use std::fs;
use std::path::{Path, PathBuf};

use crate::project::config::ProjectConfig;
use crate::project::{ItemDescriptor, Project};

pub mod html;
pub mod info;
pub mod notebook;

#[derive(Clone)]
pub struct GeneratorContext {
    pub root: PathBuf,
    pub project: Project<Option<Document<RenderResult>>>,
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

pub struct Mover;

pub struct MoveContext {
    pub project_path: PathBuf,
    pub build_dir: PathBuf,
    pub settings: ParserSettings,
}

impl Mover {
    fn create_build_path(
        content_path: PathBuf,
        build_path: PathBuf,
        entry_path: PathBuf,
    ) -> anyhow::Result<PathBuf> {
        let base_path = entry_path.strip_prefix(content_path)?;
        Ok(build_path.join(base_path))
    }

    pub fn traverse_dir(path: PathBuf, ctx: &MoveContext) -> anyhow::Result<()> {
        let content_path = ctx.project_path.join("content");

        for entry in fs::read_dir(path.as_path())? {
            let entry = entry?;
            let entry_path = entry.path();

            let metadata = fs::metadata(entry.path())?;

            if metadata.is_file() {
                let dest = Mover::create_build_path(
                    content_path.to_path_buf(),
                    ctx.build_dir.to_path_buf(),
                    entry_path.to_path_buf(),
                )?;

                let ext = entry_path.as_path().extension().unwrap().to_str().unwrap();
                match ext {
                    "md" | "ipynb" => {}
                    "py" => {
                        println!("hej");
                        let input = fs::read_to_string(entry_path.as_path())?;
                        let parsed = parse_code_string(&input)?;

                        let output = parsed.write_string(ctx.settings.solutions);

                        fs::write(dest, output)?;
                    }
                    _ => {
                        fs::copy(entry_path, dest)?;
                    }
                }
            } else {
                Mover::traverse_dir(entry_path, ctx)?;
            }
        }

        Ok(())
    }
}
