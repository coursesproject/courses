use crate::generators::{Generator, GeneratorContext};
use crate::pipeline::Mode;
use crate::project::ItemDescriptor;
use anyhow::Context;
use cdoc::config::{LaTexFormat, OutputFormat};
use cdoc::document::Document;
use cdoc::renderers::RenderResult;
use cdoc::templates::{TemplateContext, TemplateManager, TemplateType};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;
use tera::Tera;

pub struct LaTeXGenerator;

impl LaTeXGenerator {
    // pub fn new(templates: Rc<TemplateManager>) -> Self {
    //     LaTeXGenerator { templates }
    // }

    fn write_document(
        &self,
        output: String,
        doc_id: String,
        doc_path: PathBuf,
        build_dir: PathBuf,
    ) -> anyhow::Result<()> {
        let mut tex_build_dir = build_dir.join(doc_path);
        tex_build_dir.pop(); // Pop filename

        let section_build_path = tex_build_dir.join(format!("{}.tex", doc_id));

        fs::create_dir_all(tex_build_dir).context("Could not create directory")?;
        fs::write(section_build_path, output).context("writing")?;

        Ok(())
    }

    fn write_file(
        &self,
        output: String,
        build_dir: PathBuf,
        doc_name: String,
    ) -> anyhow::Result<()> {
        let file_path = build_dir.join(format!("{}.tex", doc_name));
        fs::create_dir_all(build_dir).context("Could not create directory")?;
        fs::write(file_path, output).context("writing")?;

        Ok(())
    }
}

impl Generator for LaTeXGenerator {
    fn generate(&self, ctx: &GeneratorContext) -> anyhow::Result<()> {
        // Copy resources
        let resource_path_src = ctx.root.join("resources");
        let resource_path_build_dir = ctx.build_dir.as_path().join("resources");

        fs::create_dir_all(resource_path_build_dir.as_path())?;
        let mut options = fs_extra::dir::CopyOptions::new();
        options.overwrite = true;

        fs_extra::copy_items(&[resource_path_src], ctx.build_dir.as_path(), &options)?;
        // end of resource copy

        // main_doc
        let proj = ctx.project.clone();

        let mut context = TemplateContext::new();
        context.insert("project", &proj);
        context.insert("config", &ctx.config);
        let result =
            ctx.templates
                .render("main", &LaTexFormat {}, TemplateType::Layout, &context)?;
        self.write_file(
            result,
            ctx.build_dir.as_path().to_path_buf(),
            "main".to_string(),
        )?;

        let result =
            ctx.templates
                .render("preamble", &LaTexFormat {}, TemplateType::Layout, &context)?;
        self.write_file(
            result,
            ctx.build_dir.as_path().to_path_buf(),
            "preamble".to_string(),
        )?;

        let spinner = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
        let pb = ProgressBar::new(0);
        pb.set_style(spinner);

        for item in ctx.project.clone() {
            if let Some(c) = item.doc.content.deref() {
                pb.set_message(format!("{}", item.doc.path.display()));
                pb.inc(1);
                if !(ctx.mode == Mode::Release && c.metadata.draft) {
                    // TODO: Merge with single
                    let mut context = TemplateContext::new();
                    context.insert("project", &proj); // TODO: THis is very confusing but I'm keeping it until I have a base working version of the new cdoc crate.
                    context.insert("config", &ctx.config);
                    context.insert("current_part", &item.part_id);
                    context.insert("current_chapter", &item.chapter_id);
                    context.insert("current_doc", &item.doc.id);
                    context.insert("doc", &c);
                    context.insert("mode", &ctx.mode);

                    let result = ctx.templates.render(
                        "section",
                        &LaTexFormat {},
                        TemplateType::Layout,
                        &context,
                    )?;
                    self.write_document(result, item.doc.id, item.doc.path, ctx.build_dir.clone())?;
                }
            }
        }
        pb.finish_and_clear();

        Ok(())
    }

    fn generate_single(
        &self,
        content: Document<RenderResult>,
        doc_info: ItemDescriptor<()>,
        ctx: &GeneratorContext,
    ) -> anyhow::Result<()> {
        if !(ctx.mode == Mode::Release && content.metadata.draft) {
            let proj = ctx.project.clone();
            let mut context = TemplateContext::new();
            context.insert("project", &proj); // TODO: THis is very confusing but I'm keeping it until I have a base working version of the new cdoc crate.
            context.insert("config", &ctx.config);
            context.insert("current_part", &doc_info.part_id);
            context.insert("current_chapter", &doc_info.chapter_id);
            context.insert("current_doc", &doc_info.doc.id);
            context.insert("doc", &content);
            context.insert("html", &content.content);
            context.insert("title", "Test");
            context.insert("mode", &ctx.mode);

            let result =
                ctx.templates
                    .render("section", &LaTexFormat {}, TemplateType::Layout, &context)?;

            self.write_document(
                result,
                doc_info.doc.id,
                doc_info.doc.path,
                ctx.build_dir.clone(),
            )?;
        }
        Ok(())
    }
}
