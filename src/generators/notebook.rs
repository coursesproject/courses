use cdoc::document::Document;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::ops::Deref;
use std::path::PathBuf;

use cdoc::renderers::RenderResult;

use crate::generators::{Generator, GeneratorContext};
use crate::project::config::ProjectConfig;
use crate::project::ItemDescriptor;

pub struct CodeOutputGenerator;

impl Generator for CodeOutputGenerator {
    fn generate(&self, ctx: GeneratorContext) -> anyhow::Result<()> {
        for item in ctx.project {
            if let Some(c) = item.doc.content.deref() {
                let mut notebook_build_dir = ctx.build_dir.as_path().join(&item.doc.path);
                notebook_build_dir.pop(); // Pop filename
                let notebook_build_path = notebook_build_dir.join(format!("{}.ipynb", item.doc.id));

                fs::create_dir_all(notebook_build_dir)?;
                fs::write(notebook_build_path, &c.content)?;
            }
        }

        Ok(())
    }

    fn generate_single(
        &self,
        content: Document<RenderResult>,
        doc_info: ItemDescriptor<()>,
        ctx: GeneratorContext,
    ) -> anyhow::Result<()> {
        let mut notebook_build_dir = ctx.build_dir.as_path().join(&doc_info.doc.path);
        notebook_build_dir.pop(); // Pop filename
        let notebook_build_path = notebook_build_dir.join(format!("{}.ipynb", doc_info.doc.id));

        fs::create_dir_all(notebook_build_dir)?;
        fs::write(notebook_build_path, content.content)?;
        Ok(())
    }
}
