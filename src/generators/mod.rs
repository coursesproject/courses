use crate::project::config::Mode;
use anyhow::{anyhow, Context as AContext};
use cdoc::config::Format;
use cdoc::renderers::RenderResult;
use cdoc::templates::{TemplateManager, TemplateType};
use cdoc_parser::document::Document;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};
use rayon::prelude::*;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Mutex;
use tera::Context;

use crate::project::caching::Cache;
use crate::project::config::ProjectConfig;
use crate::project::{
    ContentItem, ContentItemDescriptor, ContentResultS, ContentResultX, ProjectItemContentVec,
    ProjectItemVec,
};

/// This type is responsible for writing the final output for a given format.
/// For formats that use layouts, this is where the document content is rendered into the layout
/// template.
// #[derive(Clone)]
pub struct Generator<'a> {
    /// Project root
    pub root: PathBuf,
    /// Structured project for inclusion in layout templates.
    pub project: &'a ContentResultX,
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

        fs::create_dir_all(&html_build_dir).with_context(|| {
            format!(
                "Could not create directory at: {}",
                html_build_dir.display()
            )
        })?;

        let file = File::create(&section_build_path)
            .with_context(|| format!("{}", section_build_path.display()))?;
        let writer = BufWriter::new(file);
        Ok(writer)
    }

    /// Run the generator.
    pub fn generate(
        &self,
        bar: ProgressBar,
        project_vec: &ProjectItemContentVec,
    ) -> anyhow::Result<Vec<anyhow::Result<String>>> {
        if self.format.include_resources() {
            let resource_path_src = self.root.join("resources");
            let resource_path_build_dir = self.build_dir.as_path().join("resources");

            fs::create_dir_all(resource_path_build_dir.as_path())?;
            let mut options = fs_extra::dir::CopyOptions::new();
            options.overwrite = true;

            fs_extra::copy_items(&[&resource_path_src], self.build_dir.as_path(), &options)
                .with_context(|| {
                    format!(
                        "from {} to {}",
                        resource_path_src.display(),
                        self.build_dir.as_path().display()
                    )
                })?;
        }

        let mut base = Context::new();
        base.insert("project", &self.project);
        base.insert("config", &self.config);

        let res = project_vec
            // .iter()
            .par_iter()
            .progress_with(bar)
            .map(|item| {
                if let Some(c) = item.doc.content.deref() {
                    self.process(&base, c, item)?;
                }
                Ok(item.doc.path.to_str().unwrap().to_string())
            })
            .collect();
        Ok(res)
    }

    /// This method writes (and renders into template if applicable) a single document.
    pub fn process<T>(
        &self,
        args: &Context,
        doc: &Document<RenderResult>,
        item: &ContentItemDescriptor<T>,
    ) -> anyhow::Result<()> {
        if !(self.mode == Mode::Release && doc.meta.draft) {
            let mut writer = self
                .get_writer(&item.doc.id, &item.doc.path, item.is_section)
                .with_context(|| {
                    format!(
                        "Could not create writer for document at path: {}",
                        item.doc.path.display()
                    )
                })?;
            if let Some(layout_id) = self.format.layout() {
                let mut args = args.clone();
                args.insert("current_path", &item.path);

                args.insert("doc", &doc);
                args.insert("mode", &self.mode);

                self.templates.render(
                    &layout_id,
                    self.format.template_prefix(),
                    TemplateType::Layout,
                    &args,
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
        &mut self,
        doc: &Document<RenderResult>,
        doc_info: &ContentItemDescriptor<()>,
    ) -> anyhow::Result<()> {
        let mut base = Context::new();
        base.insert("project", &self.project);
        base.insert("config", &self.config);
        self.process(&base, doc, doc_info)
    }
}
