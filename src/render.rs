use crate::cfg::Config;
use crate::parser::DocumentParsed;
use crate::pipeline::DocumentConfig;
use anyhow::{anyhow, Context};
use std::path::{Path, PathBuf};
use tera::Tera;

pub struct HtmlRenderer {
    project_path: PathBuf,
    tera: Tera,
}

impl HtmlRenderer {
    pub fn new<P: AsRef<Path>>(project_path: P) -> anyhow::Result<Self> {
        let path_str = project_path
            .as_ref()
            .to_str()
            .ok_or(anyhow!("Invalid path"))?;
        let pattern = path_str.to_string() + "/templates/**/*.tera.html";
        Ok(HtmlRenderer {
            project_path: project_path.as_ref().to_path_buf(),
            tera: Tera::new(&pattern)?,
        })
    }

    pub fn render_document(
        &self,
        doc: &DocumentParsed,
        config: &Config<DocumentConfig>,
    ) -> anyhow::Result<String> {
        let mut context = tera::Context::new();
        context.insert("config", config);
        context.insert("current_section", "hej");
        context.insert("current_chapter", "hej");
        context.insert("html", &doc.html);
        context.insert("title", "Test");
        context.insert("meta", &doc.frontmatter);
        self.tera
            .render("section.tera.html", &context)
            .context("Render error")
    }
}
