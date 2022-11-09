use crate::cfg::{Config, ProjectConfig};
use crate::parser::DocumentParsed;
use crate::pipeline::DocumentConfig;
use crate::render::HtmlRenderError::TemplateError;
use anyhow::anyhow;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tera::Tera;
use thiserror::Error;

pub struct HtmlRenderer {
    project_path: PathBuf,
    project_config: ProjectConfig,
    tera: Tera,
}

impl HtmlRenderer {
    pub fn new<P: AsRef<Path>>(project_path: P) -> anyhow::Result<Self> {
        let path_str = project_path
            .as_ref()
            .to_str()
            .ok_or(anyhow!("Invalid path"))?;
        let pattern = path_str.to_string() + "/templates/**/*.tera.html";

        let config_path = project_path.as_ref().join("config.yml");
        let config_reader = BufReader::new(File::open(config_path)?);
        let project_config: ProjectConfig = serde_yaml::from_reader(config_reader)?;

        Ok(HtmlRenderer {
            project_path: project_path.as_ref().to_path_buf(),
            tera: Tera::new(&pattern)?,
            project_config,
        })
    }

    pub fn render_document(
        &self,
        doc: &DocumentParsed,
        config: &Config<DocumentConfig>,
    ) -> tera::Result<String> {
        let mut context = tera::Context::new();
        context.insert("config", config);
        context.insert("project", &self.project_config);
        context.insert("current_section", "hej");
        context.insert("current_chapter", "hej");
        context.insert("html", &doc.html);
        context.insert("title", "Test");
        context.insert("meta", &doc.frontmatter);
        self.tera.render("section.tera.html", &context)
    }
}

#[derive(Error, Debug)]
pub enum HtmlRenderError {
    #[error("template render error")]
    TemplateError(tera::Error, String),
    #[error("other error")]
    Other(anyhow::Error, String),
}
