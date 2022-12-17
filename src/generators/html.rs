use crate::generators::{Generator, GeneratorContext};
use anyhow::anyhow;
use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::path::Path;
use tera::Tera;
use thiserror::Error;

pub struct HtmlGenerator {
    tera: Tera,
}

impl HtmlGenerator {
    pub fn new<P: AsRef<Path>>(project_path: P) -> anyhow::Result<Self> {
        let path_str = project_path
            .as_ref()
            .to_str()
            .ok_or_else(|| anyhow!("Invalid path"))?;
        let pattern = path_str.to_string() + "/templates/**/*.tera.html";

        Ok(HtmlGenerator {
            tera: Tera::new(&pattern)?,
        })
    }
}

impl Generator for HtmlGenerator {
    fn generate(&self, ctx: GeneratorContext) -> Result<(), anyhow::Error> {
        for item in ctx.project {
            if let Some(c) = item.doc.content.deref() {
                let mut context = tera::Context::new();
                context.insert("config", &ctx.project); // TODO: THis is very confusing but I'm keeping it until I have a base working version of the new cdoc crate.
                context.insert("project", &ctx.config);
                context.insert("current_part", &item.part_id);
                context.insert("current_chapter", &item.chapter_id);
                context.insert("current_doc", &item.doc.id);
                context.insert("html", c);
                context.insert("title", "Test");

                self.tera.render("section.tera.html", &context)?;
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum HtmlRenderError {
    #[error("template render error")]
    TemplateError(tera::Error, String),
    #[error("other error")]
    Other(anyhow::Error, String),
}
