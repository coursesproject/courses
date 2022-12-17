mod html;

use crate::project::config::ProjectConfig;
use crate::project::{Project, ProjectItem};
use anyhow::anyhow;
use cdoc::config::{Format, PipelineConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct Input {
    pub content: String,
    pub format: Format,
}

pub struct GeneratorContext {
    pub root: PathBuf,
    pub project: Project<String>,
    pub config: ProjectConfig,
    pub pipeline: &PipelineConfig,
    pub build_path: PathBuf,
}

pub struct Generator {
    format: Format,
    ctx: GeneratorContext,
}

impl Generator {
    pub fn new(ctx: GeneratorContext, format: Format) -> Self {
        Generator { ctx, format }
    }

    pub fn generate(&self) -> Result<(), Box<dyn std::error::Error>> {
        for item in self.ctx.project {
            self.generate_single(&item)?;
        }
        Ok(())
    }

    pub fn generate_single(
        &self,
        item: &ProjectItem<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let doc = self
            .ctx
            .pipeline
            .loaders
            .get(&item.doc.content.format)
            .ok_or(anyhow!("No spect found"))?
            .load(&item.doc.content.content)?;

        if doc.metadata.output.contains(&self.format) {
            let res = self
                .ctx
                .pipeline
                .parsers
                .get(&self.format)
                .ok_or(anyhow!("No spec found"))?
                .parse(&doc, &HashMap::new())?;
            let output = self
                .ctx
                .pipeline
                .renderers
                .get(&self.format)
                .ok_or(anyhow!("No spec found"))?
                .render(&res);

            let mut build_dir = self
                .ctx
                .build_path
                .as_path()
                .join(&self.format)
                .join(&item.doc.path);
            build_dir.pop(); // Pop filename so only directory remains

            let file_path = build_dir.join(format!("{}.{}", item.doc.id, self.format));
            fs::create_dir_all(build_dir)?;
            fs::write(file_path, output)?;
        }
        Ok(())
    }
}
