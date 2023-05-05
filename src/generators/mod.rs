use crate::pipeline::Mode;
use anyhow::Context as AContext;
use cdoc::config::Format;
use cdoc::document::Document;
use cdoc::renderers::RenderResult;
use cdoc::templates::{TemplateManager, TemplateType};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;
use tera::Context;

use crate::project::config::ProjectConfig;
use crate::project::{ItemDescriptor, ProjectItemVec, ProjectResult};

#[derive(Clone)]
pub struct Generator<'a> {
    pub root: PathBuf,
    pub project_vec: &'a ProjectItemVec,
    pub project: ProjectResult<'a>,
    pub templates: &'a TemplateManager,
    pub config: ProjectConfig,
    pub mode: Mode,
    pub build_dir: PathBuf,
    pub format: &'a dyn Format,
}

impl Generator<'_> {
    fn get_writer(&self, doc_id: &str, doc_path: &PathBuf) -> anyhow::Result<impl Write> {
        let mut html_build_dir = self.build_dir.join(doc_path);
        html_build_dir.pop(); // Pop filename

        let section_build_path =
            html_build_dir.join(format!("{}.{}", doc_id, self.format.extension()));

        fs::create_dir_all(html_build_dir).context("Could not create directory")?;

        let file = File::create(section_build_path)?;
        let writer = BufWriter::new(file);
        Ok(writer)
    }
    fn write_document(&self, output: &str, doc_id: &str, doc_path: &PathBuf) -> anyhow::Result<()> {
        let mut html_build_dir = self.build_dir.join(doc_path);
        html_build_dir.pop(); // Pop filename

        let section_build_path =
            html_build_dir.join(format!("{}.{}", doc_id, self.format.extension()));

        fs::create_dir_all(html_build_dir).context("Could not create directory")?;

        fs::write(section_build_path, output).context("writing")?;

        Ok(())
    }

    pub fn generate(&self, bar: ProgressBar) -> anyhow::Result<()> {
        if self.format.include_resources() {
            let resource_path_src = self.root.join("resources");
            let resource_path_build_dir = self.build_dir.as_path().join("resources");

            fs::create_dir_all(resource_path_build_dir.as_path())?;
            let mut options = fs_extra::dir::CopyOptions::new();
            options.overwrite = true;

            fs_extra::copy_items(&[resource_path_src], self.build_dir.as_path(), &options)?;
        }

        let res: anyhow::Result<()> = self
            .project_vec
            .par_iter()
            .progress_with(bar)
            .map(|item| {
                if let Some(c) = item.doc.content.deref() {
                    self.process(c, item)?;
                }
                Ok(())
            })
            .collect();
        res?;

        Ok(())
    }

    pub fn process<T>(
        &self,
        doc: &Document<RenderResult>,
        item: &ItemDescriptor<T>,
    ) -> anyhow::Result<()> {
        if !(self.mode == Mode::Release && doc.metadata.draft) {
            let mut writer = self.get_writer(&item.doc.id, &item.doc.path)?;
            if self.format.use_layout() {
                let mut context = Context::default();
                context.insert("project", &self.project); // TODO: THis is very confusing but I'm keeping it until I have a base working version of the new cdoc crate.
                context.insert("config", &self.config);
                context.insert("current_part", &item.part_id);
                context.insert("current_chapter", &item.chapter_id);
                context.insert("current_doc", &item.doc.id);
                context.insert("doc", &doc);
                context.insert("mode", &self.mode);

                self.templates.render(
                    "section",
                    self.format,
                    TemplateType::Layout,
                    &context,
                    &mut writer,
                )?;
            } else {
                writer.write(doc.content.as_bytes())?;
            };
        }
        Ok(())
    }

    pub fn generate_single(
        &self,
        doc: &Document<RenderResult>,
        doc_info: &ItemDescriptor<()>,
    ) -> anyhow::Result<()> {
        self.process(doc, doc_info)
    }
}
