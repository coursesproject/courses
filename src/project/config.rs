use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use cdoc::config::Format;

use cdoc::package::Dependency;
use cdoc::parser::{Parser, ParserSettings};

use cdoc::preprocessors::cell_outputs::CellOutputConfig;
use cdoc::preprocessors::AstPreprocessorConfig;
use cdoc::renderers::extensions::structure::DocStructureConfig;
use cdoc::renderers::extensions::RenderExtensionConfig;
use cdoc_parser::notebook::NotebookMeta;
use clap::ValueEnum;
use semver::VersionReq;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfigDummy {
    pub courses: CoursesConfig,

    #[serde(flatten)]
    pub everything: HashMap<String, serde_yaml::Value>,
}

/// Refers to a configuration.yml file in the project that specifies a variety
/// of options for the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub courses: CoursesConfig,

    #[serde(default)]
    pub url_prefix: String,
    #[serde(default)]
    pub repository: RepositoryConfig,
    #[serde(default)]
    pub outputs: Vec<Box<dyn Format>>,
    #[serde(default = "default_profiles")]
    pub profiles: HashMap<String, Profile>,

    #[serde(default)]
    pub packages: Vec<Dependency>,

    #[serde(default)]
    pub custom: HashMap<String, serde_yaml::Value>,
    pub notebook_meta: Option<NotebookMeta>,

    #[serde(default)]
    pub scripts: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoursesConfig {
    pub version: VersionReq,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    #[serde(default)]
    pub mode: Mode,
    #[serde(default)]
    pub parser: Parser,
    #[serde(default)]
    pub formats: Vec<Box<dyn Format>>,
    #[serde(default)]
    pub render_extensions: Vec<Box<dyn RenderExtensionConfig>>,
    #[serde(default)]
    pub create_filters: bool,
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

fn default_profiles() -> HashMap<String, Profile> {
    let mut p = HashMap::new();
    p.insert(
        "draft".to_string(),
        Profile {
            mode: Mode::Draft,
            parser: Parser {
                preprocessors: vec![Box::new(CellOutputConfig) as Box<dyn AstPreprocessorConfig>],
                settings: ParserSettings { solutions: true },
            },
            formats: vec![],
            render_extensions: vec![Box::new(DocStructureConfig {
                max_heading_level: 0,
                included_commands: vec![],
            })],
            create_filters: true,
        },
    );

    p.insert(
        "release".to_string(),
        Profile {
            mode: Mode::Release,
            parser: Parser {
                preprocessors: vec![Box::new(CellOutputConfig) as Box<dyn AstPreprocessorConfig>],
                settings: ParserSettings { solutions: false },
            },
            formats: vec![],
            render_extensions: vec![Box::new(DocStructureConfig {
                max_heading_level: 0,
                included_commands: vec![],
            })],
            create_filters: false,
        },
    );

    p
}

/// Build mode. This is used internally for generation but is also available in templates.
#[derive(Serialize, Deserialize, Clone, Debug, Copy, ValueEnum, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Don't include drafts
    Release,
    /// Include drafts.
    Draft,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Draft
    }
}
