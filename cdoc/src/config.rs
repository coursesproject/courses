use std::cmp::{Eq, PartialEq};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

use anyhow::anyhow;
use clap::ValueEnum;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

use crate::loader::{Loader, MarkdownLoader, NotebookLoader};

use crate::renderers::generic::GenericRenderer;
use crate::renderers::notebook::NotebookRenderer;
use crate::renderers::DocumentRenderer;

/// Input formats. Currently supports regular markdown files as well as Jupyter Notebooks.
#[derive(Hash, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Debug, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum InputFormat {
    Markdown,
    Notebook,
}

/// Implementors define a format. This trait should make format extensions easy to implement.
#[typetag::serde]
pub trait Format: DynClone + Debug + Send + Sync {
    /// Return the file extension used for the given format.
    fn extension(&self) -> &str;
    /// Template format name. Useful if templates are reused across formats as is the case for
    /// notebooks which use markdown.
    fn template_prefix(&self) -> &str;
    /// Format name that is used in status messages, build output and in the configuration file.
    fn name(&self) -> &str;
    /// Return true if the format should not be parsed. This may be removed in the future and is
    /// currently only used for the info format which exports all parsed contents in a project.
    fn no_parse(&self) -> bool;
    /// Return a renderer instance. Currently does not allow for configuration.
    fn renderer(&self) -> Box<dyn DocumentRenderer>;
    /// Determines whether non-source files should be copied to
    fn include_resources(&self) -> bool;
    fn layout(&self) -> Option<String>;
}

impl PartialEq for dyn Format {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Hash for dyn Format {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state)
    }
}

impl Eq for dyn Format {}

// impl Eq for Box<dyn Format> {}

impl Display for dyn Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

dyn_clone::clone_trait_object!(Format);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotebookFormat {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HtmlFormat {}

/// Used to produce an output yml file containing all sources and metadata in a single file
/// structured like the content folder.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InfoFormat {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarkdownFormat {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LaTexFormat {}

/// Custom output format definition. It should be possible to create almost any text-based output.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DynamicFormat {
    /// Output file extension
    pub extension: String,
    /// Template prefix (used in template files)
    pub template_prefix: String,
    /// Format name (used for build folder)
    pub name: String,
    /// Renderer to use (generic or notebook)
    #[serde(default = "default_renderer")]
    pub renderer: Box<dyn DocumentRenderer>,
    /// Include resources folder in output
    #[serde(default)]
    pub include_resources: bool,
    /// Use layout template
    pub layout: Option<String>,
}

fn default_renderer() -> Box<dyn DocumentRenderer> {
    Box::<GenericRenderer>::default()
}

#[typetag::serde(name = "dynamic")]
impl Format for DynamicFormat {
    fn extension(&self) -> &str {
        &self.extension
    }

    fn template_prefix(&self) -> &str {
        &self.template_prefix
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn no_parse(&self) -> bool {
        false
    }

    fn renderer(&self) -> Box<dyn DocumentRenderer> {
        self.renderer.clone()
    }

    fn include_resources(&self) -> bool {
        self.include_resources
    }

    fn layout(&self) -> Option<String> {
        self.layout.clone()
    }
}

#[typetag::serde(name = "notebook")]
impl Format for NotebookFormat {
    fn extension(&self) -> &str {
        "ipynb"
    }

    fn template_prefix(&self) -> &str {
        "markdown"
    }

    fn name(&self) -> &str {
        "notebook"
    }

    fn no_parse(&self) -> bool {
        false
    }

    fn renderer(&self) -> Box<dyn DocumentRenderer> {
        Box::new(NotebookRenderer)
    }

    fn include_resources(&self) -> bool {
        true
    }

    fn layout(&self) -> Option<String> {
        None
    }
}

#[typetag::serde(name = "html")]
impl Format for HtmlFormat {
    fn extension(&self) -> &str {
        "html"
    }

    fn template_prefix(&self) -> &str {
        "html"
    }

    fn name(&self) -> &str {
        "html"
    }

    fn no_parse(&self) -> bool {
        false
    }

    fn renderer(&self) -> Box<dyn DocumentRenderer> {
        Box::<GenericRenderer>::default()
    }
    fn include_resources(&self) -> bool {
        true
    }
    fn layout(&self) -> Option<String> {
        Some("section".to_string())
    }
}

#[typetag::serde(name = "info")]
impl Format for InfoFormat {
    fn extension(&self) -> &str {
        "yml"
    }

    fn template_prefix(&self) -> &str {
        "yml"
    }

    fn name(&self) -> &str {
        "info"
    }

    fn no_parse(&self) -> bool {
        true
    }

    fn renderer(&self) -> Box<dyn DocumentRenderer> {
        Box::<GenericRenderer>::default()
    }
    fn include_resources(&self) -> bool {
        false
    }
    fn layout(&self) -> Option<String> {
        None
    }
}

#[typetag::serde(name = "markdown")]
impl Format for MarkdownFormat {
    fn extension(&self) -> &str {
        "md"
    }

    fn template_prefix(&self) -> &str {
        "markdown"
    }

    fn name(&self) -> &str {
        "markdown"
    }

    fn no_parse(&self) -> bool {
        false
    }
    fn renderer(&self) -> Box<dyn DocumentRenderer> {
        Box::<GenericRenderer>::default()
    }
    fn include_resources(&self) -> bool {
        false
    }
    fn layout(&self) -> Option<String> {
        None
    }
}

#[typetag::serde(name = "latex")]
impl Format for LaTexFormat {
    fn extension(&self) -> &str {
        "tex"
    }

    fn template_prefix(&self) -> &str {
        "latex"
    }

    fn name(&self) -> &str {
        "latex"
    }

    fn no_parse(&self) -> bool {
        false
    }
    fn renderer(&self) -> Box<dyn DocumentRenderer> {
        Box::<GenericRenderer>::default()
    }
    fn include_resources(&self) -> bool {
        true
    }
    fn layout(&self) -> Option<String> {
        Some("section".to_string())
    }
}

impl InputFormat {
    /// Get loader for format (designed to be extensible)
    pub fn loader(&self) -> Box<dyn Loader> {
        match self {
            InputFormat::Markdown => Box::new(MarkdownLoader),
            InputFormat::Notebook => Box::new(NotebookLoader),
        }
    }

    /// Format extension
    pub fn extension(&self) -> &str {
        match self {
            InputFormat::Markdown => "md",
            InputFormat::Notebook => "ipynb",
        }
    }

    /// Name can be used by tools like courses to display the current format
    pub fn name(&self) -> &str {
        match self {
            InputFormat::Markdown => "markdown",
            InputFormat::Notebook => "notebook",
        }
    }

    pub fn from_extension(ext: &str) -> Result<Self, anyhow::Error> {
        match ext {
            "md" => Ok(InputFormat::Markdown),
            "ipynb" => Ok(InputFormat::Notebook),
            _ => Err(anyhow!("Invalid extension for input")),
        }
    }

    pub fn from_name(name: &str) -> Result<Self, anyhow::Error> {
        match name {
            "markdown" => Ok(InputFormat::Markdown),
            "notebook" => Ok(InputFormat::Notebook),
            _ => Err(anyhow!("Invalid format name for input")),
        }
    }
}

impl Display for InputFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
