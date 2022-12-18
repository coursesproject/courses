use std::collections::HashMap;
use anyhow::anyhow;
use cdoc::config::OutputFormat;
use serde::{Deserialize, Serialize};
use cdoc::parser::Parser;

/// Refers to a configuration.yml file in the project that specifies a variety
/// of options for the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub url_prefix: String,
    #[serde(default)]
    pub build: BuildConfigSet,
    pub outputs: Vec<OutputFormat>,
    pub parsers: HashMap<OutputFormat, Parser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfigSet {
    pub dev: BuildConfig,
    pub release: BuildConfig,
}

impl BuildConfigSet {
    pub fn get_config(&self, mode: &str) -> anyhow::Result<BuildConfig> {
        match mode {
            "dev" => Ok(self.dev.clone()),
            "release" => Ok(self.release.clone()),
            _ => Err(anyhow!("Invalid build mode")),
        }
    }
}

impl Default for BuildConfigSet {
    fn default() -> Self {
        BuildConfigSet {
            dev: BuildConfig {
                katex_output: false,
            },
            release: BuildConfig { katex_output: true },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub katex_output: bool,
}
