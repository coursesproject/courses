use std::cmp::{Eq, PartialEq};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

use anyhow::anyhow;
use clap::ValueEnum;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

use crate::loader::{Loader, MarkdownLoader, NotebookLoader};
use crate::parser::{Parser, ParserSettings};
use crate::processors::exercises::ExercisesConfig;
// use crate::renderers::html::HtmlRenderer;
// use crate::renderers::latex::LatexRenderer;
// use crate::renderers::markdown::MarkdownRenderer;
// use crate::renderers::notebook::NotebookRenderer;
use crate::renderers::generic::GenericRenderer;
use crate::renderers::notebook::NotebookRenderer;
use crate::renderers::DocumentRenderer;

#[derive(Hash, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Debug, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum InputFormat {
    Markdown,
    Notebook,
}

#[typetag::serde]
pub trait Format: DynClone + Debug + Send + Sync {
    fn extension(&self) -> &str;
    fn template_name(&self) -> &str;
    fn name(&self) -> &str;
    fn no_parse(&self) -> bool;
    fn renderer(&self) -> Box<dyn DocumentRenderer>;
    fn include_resources(&self) -> bool;
    fn use_layout(&self) -> bool;
}

impl PartialEq for dyn Format {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

// impl<'a> Borrow<dyn Format + 'a> for dyn Format {
//     fn borrow(&self) -> &(dyn Format + 'a) {
//         self
//     }
// }

// impl PartialEq for Box<dyn Format> {
//     fn eq(&self, other: &Self) -> bool {
//         self.name() == other.name()
//     }
// }

// impl Hash for Box<dyn Format> {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.name().hash(state)
//     }
// }

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InfoFormat {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarkdownFormat {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LaTexFormat {}

#[typetag::serde(name = "notebook")]
impl Format for NotebookFormat {
    fn extension(&self) -> &str {
        "ipynb"
    }

    fn template_name(&self) -> &str {
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
        false
    }

    fn use_layout(&self) -> bool {
        false
    }
}

#[typetag::serde(name = "html")]
impl Format for HtmlFormat {
    fn extension(&self) -> &str {
        "html"
    }

    fn template_name(&self) -> &str {
        "html"
    }

    fn name(&self) -> &str {
        "html"
    }

    fn no_parse(&self) -> bool {
        false
    }

    fn renderer(&self) -> Box<dyn DocumentRenderer> {
        Box::new(GenericRenderer)
    }
    fn include_resources(&self) -> bool {
        true
    }
    fn use_layout(&self) -> bool {
        true
    }
}

#[typetag::serde(name = "info")]
impl Format for InfoFormat {
    fn extension(&self) -> &str {
        "yml"
    }

    fn template_name(&self) -> &str {
        "yml"
    }

    fn name(&self) -> &str {
        "info"
    }

    fn no_parse(&self) -> bool {
        true
    }

    fn renderer(&self) -> Box<dyn DocumentRenderer> {
        Box::new(GenericRenderer)
    }
    fn include_resources(&self) -> bool {
        false
    }
    fn use_layout(&self) -> bool {
        false
    }
}

#[typetag::serde(name = "markdown")]
impl Format for MarkdownFormat {
    fn extension(&self) -> &str {
        "md"
    }

    fn template_name(&self) -> &str {
        "markdown"
    }

    fn name(&self) -> &str {
        "markdown"
    }

    fn no_parse(&self) -> bool {
        false
    }
    fn renderer(&self) -> Box<dyn DocumentRenderer> {
        Box::new(GenericRenderer)
    }
    fn include_resources(&self) -> bool {
        false
    }
    fn use_layout(&self) -> bool {
        false
    }
}

#[typetag::serde(name = "latex")]
impl Format for LaTexFormat {
    fn extension(&self) -> &str {
        "tex"
    }

    fn template_name(&self) -> &str {
        "latex"
    }

    fn name(&self) -> &str {
        "latex"
    }

    fn no_parse(&self) -> bool {
        false
    }
    fn renderer(&self) -> Box<dyn DocumentRenderer> {
        Box::new(GenericRenderer)
    }
    fn include_resources(&self) -> bool {
        true
    }
    fn use_layout(&self) -> bool {
        true
    }
}

#[derive(Hash, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Notebook,
    Html,
    Info,
    Markdown,
    LaTeX,
}

impl InputFormat {
    pub fn loader(&self) -> Box<dyn Loader> {
        match self {
            InputFormat::Markdown => Box::new(MarkdownLoader),
            InputFormat::Notebook => Box::new(NotebookLoader),
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            InputFormat::Markdown => "md",
            InputFormat::Notebook => "ipynb",
        }
    }

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

impl OutputFormat {
    pub fn no_parse(&self) -> bool {
        match self {
            OutputFormat::Notebook => false,
            OutputFormat::Html => false,
            OutputFormat::Info => true,
            OutputFormat::LaTeX => false,
            OutputFormat::Markdown => false,
        }
    }

    // pub fn from_extension(ext: &str) -> Result<Self, anyhow::Error> {
    //     match ext {
    //         "ipynb" => Ok(OutputFormat::Notebook),
    //         "html" => Ok(OutputFormat::Html),
    //         _ => Err(anyhow!("Invalid extension for output")),
    //     }
    // }

    // pub fn from_name(name: &str) -> Result<Self, anyhow::Error> {
    //     match name {
    //         "notebook" => Ok(OutputFormat::Notebook),
    //         "html" => Ok(OutputFormat::Html),
    //         "info" => Ok(OutputFormat::Info),
    //         _ => Err(anyhow!("Invalid format name for output")),
    //     }
    // }

    pub fn extension(&self) -> &str {
        match self {
            OutputFormat::Notebook => "ipynb",
            OutputFormat::Html => "html",
            OutputFormat::Info => "yml",
            OutputFormat::LaTeX => "tex",
            OutputFormat::Markdown => "md",
        }
    }

    pub fn template_extension(&self) -> &str {
        match self {
            OutputFormat::Notebook => "md",
            OutputFormat::Html => "html",
            OutputFormat::Info => "yml",
            OutputFormat::LaTeX => "tex",
            OutputFormat::Markdown => "md",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            OutputFormat::Notebook => "notebook",
            OutputFormat::Html => "html",
            OutputFormat::Info => "info",
            OutputFormat::LaTeX => "latex",
            OutputFormat::Markdown => "markdown",
        }
    }
}

impl Display for InputFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[allow(unused)]
fn get_default_parser(_format: OutputFormat) -> Parser {
    Parser {
        preprocessors: vec![Box::new(ExercisesConfig)],
        settings: ParserSettings {
            solutions: false,
            notebook_outputs: false,
        },
    }
}
