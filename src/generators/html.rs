use std::fs;
use std::ops::Deref;
use std::path::PathBuf;

use anyhow::Context;
use tera::Tera;

use cdoc::document::Document;
use cdoc::renderers::RenderResult;

use crate::generators::{Generator, GeneratorContext};
use crate::project::ItemDescriptor;

pub struct HtmlGenerator {
    tera: Tera,
}

impl HtmlGenerator {
    pub fn new(tera: Tera) -> Self {
        HtmlGenerator { tera }
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
        let mut html_build_dir = build_dir.join(doc_path);
        html_build_dir.pop(); // Pop filename

        let section_build_path = html_build_dir.join(format!("{}.html", doc_id));

        fs::create_dir_all(html_build_dir).context("Could not create directory")?;
        fs::write(section_build_path, output).unwrap();

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

        let resource_path_src = ctx.root.join("resources");
        let resource_path_build_dir = ctx.build_dir.join("resources");

        fs::create_dir_all(resource_path_build_dir.as_path())?;
        fs_extra::copy_items(
            &[resource_path_src],
            ctx.build_dir,
            &fs_extra::dir::CopyOptions::default(),
        )?;

        Ok(())
    }

    fn generate_single(
        &self,
        content: Document<RenderResult>,
        doc_info: ItemDescriptor<()>,
        ctx: GeneratorContext,
    ) -> anyhow::Result<()> {
        let proj = ctx.project.clone();
        let mut context = tera::Context::new();
        context.insert("config", &proj); // TODO: THis is very confusing but I'm keeping it until I have a base working version of the new cdoc crate.
        context.insert("project", &ctx.config);
        context.insert("current_part", &doc_info.part_id);
        context.insert("current_chapter", &doc_info.chapter_id);
        context.insert("current_doc", &doc_info.doc.id);
        context.insert("html", &content.content);
        context.insert("title", "Test");

        let result = self.tera.render("section.tera.html", &context)?;
        self.write_document(result, doc_info.doc.id, doc_info.doc.path, ctx.build_dir)?;

        Ok(())
    }
}
