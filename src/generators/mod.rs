mod html;

use crate::project::config::ProjectConfig;
use crate::project::{Project, ProjectItem};
use anyhow::anyhow;
use cdoc::config::{Format, PipelineConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;



pub struct GeneratorContext<'a> {
    pub root: PathBuf,
    pub project: Project<Option<String>>,
    pub config: ProjectConfig,
    pub pipeline: &'a PipelineConfig,
    pub build_path: PathBuf,
}

pub trait Generator {
    fn generate(&self, ctx: GeneratorContext) -> Result<(), anyhow::Error>;
    // fn generate_single(&self);
}
