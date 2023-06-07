use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use cdoc::config::Format;
use cdoc::notebook::NotebookMeta;
use cdoc::parser::Parser;

/// Refers to a configuration.yml file in the project that specifies a variety
/// of options for the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub url_prefix: String,
    #[serde(default)]
    pub repository: RepositoryConfig,
    pub outputs: Vec<Box<dyn Format>>,
    // pub profiles: HashMap<String, Parser>,
    #[serde(flatten)]
    pub parser: Parser,
    #[serde(default)]
    pub custom: HashMap<String, serde_yaml::Value>,
    pub notebook_meta: Option<NotebookMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepositoryConfig {
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfigSet {
    pub dev: BuildConfig,
    pub release: BuildConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub katex_output: bool,
}
