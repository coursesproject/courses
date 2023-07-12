use crate::project::config::Mode;
use anyhow::{anyhow, Context as AContext};
use cdoc::config::Format;
use cdoc::document::Document;
use cdoc::renderers::RenderResult;
use cdoc::templates::{TemplateManager, TemplateType};
use indicatif::{ParallelProgressIterator, ProgressBar};
use rayon::prelude::*;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::Deref;
use std::path::PathBuf;
use tera::Context;

use crate::project::config::ProjectConfig;
use crate::project::{ContentItemDescriptor, ContentResultS, ProjectItemVec};

/// This type is responsible for writing the final output for a given format.
/// For formats that use layouts, this is where the document content is rendered into the layout
/// template.
#[derive(Clone)]
pub struct Generator<'a> {
    /// Project root
    pub root: PathBuf,
    /// Rendered content as a vector. This format is used to enable parallelization.
    pub project_vec: &'a ProjectItemVec,
    /// Structured project for inclusion in layout templates.
    pub project: ContentResultS,
    /// Template manager is used to render the layout.
    pub templates: &'a TemplateManager,
    /// The project configuration is included in template contexts.
    pub config: ProjectConfig,
    /// Mode toggle to enable/disable draft inclusion.
    pub mode: Mode,
    /// Build dir (relative to project root).
    pub build_dir: PathBuf,
    /// Output format (used to determine whether to use layout and for the template manager).
    pub format: &'a dyn Format,
}

impl Generator<'_> {
    fn get_writer(
        &self,
        doc_id: &str,
        doc_path: &PathBuf,
        is_section: bool,
    ) -> anyhow::Result<impl Write> {
        let relative_doc_path = doc_path
            .strip_prefix(self.root.join("content").as_path())
            .unwrap_or(doc_path);
        let mut html_build_dir = self.build_dir.join(relative_doc_path);
        html_build_dir.pop(); // Pop filename

        let id = if is_section { "index" } else { doc_id };
        let section_build_path = html_build_dir.join(format!("{}.{}", id, self.format.extension()));

        // println!("sec path: {}", section_build_path.display());

        fs::create_dir_all(html_build_dir).context("Could not create directory")?;

        let file = File::create(section_build_path)?;
        let writer = BufWriter::new(file);
        Ok(writer)
    }

    /// Run the generator.
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

    /// This method writes (and renders into template if applicable) a single document.
    pub fn process<T>(
        &self,
        doc: &Document<RenderResult>,
        item: &ContentItemDescriptor<T>,
    ) -> anyhow::Result<()> {
        if !(self.mode == Mode::Release && doc.metadata.draft) {
            let mut writer = self.get_writer(&item.doc.id, &item.doc.path, item.is_section)?;
            if self.format.use_layout() {
                let mut context = Context::default();
                context.insert("project", &self.project); // TODO: THis is very confusing but I'm keeping it until I have a base working version of the new cdoc crate.
                context.insert("config", &self.config);
                context.insert("current_path", &item.path);
                // context.insert("current_part", &item.part_id);
                // context.insert("current_chapter", &item.chapter_id);
                // context.insert("current_doc", &item.doc.id);
                context.insert("doc", &doc);
                context.insert("mode", &self.mode);

                self.templates.render(
                    "section",
                    self.format.template_prefix(),
                    TemplateType::Layout,
                    &context,
                    &mut writer,
                )?;
            } else {
                let bytes = doc.content.as_bytes();
                let l = writer.write(bytes)?;
                (l == bytes.len())
                    .then_some(())
                    .ok_or(anyhow!("did not write the correct amount of bytes"))?;
            };
        }
        Ok(())
    }

    /// Function that writes only a single file.
    pub fn generate_single(
        &self,
        doc: &Document<RenderResult>,
        doc_info: &ContentItemDescriptor<()>,
    ) -> anyhow::Result<()> {
        self.process(doc, doc_info)
    }
}
