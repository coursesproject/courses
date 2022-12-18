use std::fs;
use std::ops::Deref;
use std::path::PathBuf;

use anyhow::Context;
use tera::Tera;

use cdoc::renderers::RenderResult;

use crate::generators::{Generator, GeneratorContext};
use crate::project::config::ProjectConfig;
use crate::project::{Project, ProjectItem};

pub struct HtmlGenerator {
    tera: Tera,
    project_config: Project<()>,
}

impl HtmlGenerator {
    pub fn new(tera: Tera, project: Project<()>) -> Self {
        HtmlGenerator {
            tera,
            project_config: project,
        }
    }
}

impl HtmlGenerator {
    fn write_document(
        &self,
        output: String,
        doc_id: String,
        doc_path: PathBuf,
        build_dir: PathBuf,
    ) -> anyhow::Result<()> {
        let mut html_build_dir = build_dir.join(&doc_path);
        html_build_dir.pop(); // Pop filename

        let section_build_path = html_build_dir.join(format!("{}.html", doc_id));

        fs::create_dir_all(html_build_dir).context("Could not create directory")?;
        fs::write(section_build_path, &output).unwrap();

        Ok(())
    }
}

impl Generator for HtmlGenerator {
    fn generate(&self, ctx: GeneratorContext) -> anyhow::Result<()> {
        let proj = ctx.project.clone();
        for item in ctx.project {
            if let Some(c) = item.doc.content.deref() {
                // TODO: Merge with single
                let mut context = tera::Context::new();
                context.insert("config", &proj); // TODO: THis is very confusing but I'm keeping it until I have a base working version of the new cdoc crate.
                context.insert("project", &ctx.config);
                context.insert("current_part", &item.part_id);
                context.insert("current_chapter", &item.chapter_id);
                context.insert("current_doc", &item.doc.id);
                context.insert("html", &c.content);
                context.insert("title", "Test");

                let result = self.tera.render("section.tera.html", &context)?;

                self.write_document(result, item.doc.id, item.doc.path, ctx.build_dir.clone())?;
            }
        }
        Ok(())
    }

    fn generate_single(
        &self,
        content: RenderResult,
        doc_info: ProjectItem<()>,
        config: ProjectConfig,
        build_dir: PathBuf,
    ) -> anyhow::Result<()> {
        let proj = self.project_config.clone();
        let mut context = tera::Context::new();
        context.insert("config", &proj); // TODO: THis is very confusing but I'm keeping it until I have a base working version of the new cdoc crate.
        context.insert("project", &config);
        context.insert("current_part", &doc_info.part_id);
        context.insert("current_chapter", &doc_info.chapter_id);
        context.insert("current_doc", &doc_info.doc.id);
        context.insert("html", &content.content);
        context.insert("title", "Test");

        let result = self.tera.render("section.tera.html", &context)?;
        self.write_document(result, doc_info.doc.id, doc_info.doc.path, build_dir)?;

        Ok(())
    }
}
