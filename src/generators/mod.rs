use crate::pipeline::Mode;
use anyhow::Context;
use cdoc::config::Format;
use cdoc::document::Document;
use cdoc::renderers::RenderResult;
use cdoc::templates::{TemplateContext, TemplateManager, TemplateType};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;

use crate::project::config::ProjectConfig;
use crate::project::{ItemDescriptor, ProjectResult};

#[derive(Clone)]
pub struct Generator<'a> {
    pub root: PathBuf,
    pub project: ProjectResult,
    pub templates: &'a TemplateManager,
    pub config: ProjectConfig,
    pub mode: Mode,
    pub build_dir: PathBuf,
    pub format: &'a dyn Format,
}

impl Generator<'_> {
    fn write_document(&self, output: &str, doc_id: &str, doc_path: &PathBuf) -> anyhow::Result<()> {
        let mut html_build_dir = self.build_dir.join(doc_path);
        html_build_dir.pop(); // Pop filename

        let section_build_path =
            html_build_dir.join(format!("{}.{}", doc_id, self.format.extension()));

        fs::create_dir_all(html_build_dir).context("Could not create directory")?;

        fs::write(section_build_path, output).context("writing")?;

        Ok(())
    }

    pub fn generate(&self) -> anyhow::Result<()> {
        if self.format.include_resources() {
            let resource_path_src = self.root.join("resources");
            let resource_path_build_dir = self.build_dir.as_path().join("resources");

            fs::create_dir_all(resource_path_build_dir.as_path())?;
            let mut options = fs_extra::dir::CopyOptions::new();
            options.overwrite = true;

            fs_extra::copy_items(&[resource_path_src], self.build_dir.as_path(), &options)?;
        }

        let spinner = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
        let pb = ProgressBar::new(0);
        pb.set_style(spinner);

        for item in self.project.clone() {
            if let Some(c) = item.doc.content.deref() {
                pb.set_message(format!("{}", item.doc.path.display()));
                pb.inc(1);
                self.process(c, &item.clone().map(|_| Ok(()))?)?;
            }
        }
        pb.finish_and_clear();

        Ok(())
    }

    pub fn process(
        &self,
        doc: &Document<RenderResult>,
        item: &ItemDescriptor<()>,
    ) -> anyhow::Result<()> {
        if !(self.mode == Mode::Release && doc.metadata.draft) {
            // TODO: Merge with single
            let result = if self.format.use_layout() {
                let mut context = TemplateContext::default();
                context.insert("project", &self.project); // TODO: THis is very confusing but I'm keeping it until I have a base working version of the new cdoc crate.
                context.insert("config", &self.config);
                context.insert("current_part", &item.part_id);
                context.insert("current_chapter", &item.chapter_id);
                context.insert("current_doc", &item.doc.id);
                context.insert("doc", &doc);
                context.insert("mode", &self.mode);

                self.templates
                    .render("section", self.format, TemplateType::Layout, &context)?
            } else {
                doc.content.clone()
            };

            self.write_document(&result, &item.doc.id, &item.doc.path)?;
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
