use crate::loader::{Loader, MarkdownLoader, NotebookLoader};
use crate::parser::Parser;
use crate::processors::code_split::CodeSplit;
use crate::processors::katex::KaTeXPreprocessor;
use crate::processors::shortcode_extender::ShortCodeProcessor;
use crate::renderers::html::HtmlRenderer;
use crate::renderers::markdown::MarkdownRenderer;
use crate::renderers::notebook::NotebookRenderer;
use crate::renderers::Renderer;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Markdown,
    Notebook,
    Html,
    LaTeX,
}

impl Format {
    pub fn from_extension(ext: &str) -> Result<Self, anyhow::Error> {
        match ext {
            "md" => Ok(Format::Markdown),
            "ipynb" => Ok(Format::Notebook),
            "html" => Ok(Format::Html),
            "latex" => Ok(Format::LaTeX),
            _ => Err(anyhow!("Extension not recognized")),
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Format::Markdown => "md",
            Format::Notebook => "ipynb",
            Format::Html => "html",
            Format::LaTeX => "latex",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Format::Markdown => "markdown",
            Format::Notebook => "notebook",
            Format::Html => "html",
            Format::LaTeX => "latex",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PipelineConfig {
    #[serde(default = "default_loaders")]
    pub loaders: HashMap<Format, Box<dyn Loader>>,
    #[serde(default = "default_renderers")]
    pub renderers: HashMap<Format, Box<dyn Renderer>>,
    #[serde(default = "default_parsers")]
    pub parsers: HashMap<Format, Box<Parser>>,
}

fn default_loaders() -> HashMap<Format, Box<dyn Loader>> {
    let mut map: HashMap<Format, Box<dyn Loader>> = HashMap::new();
    map.insert(Format::Markdown, Box::new(MarkdownLoader));
    map.insert(Format::Notebook, Box::new(NotebookLoader));
    map
}

fn default_renderers() -> HashMap<Format, Box<dyn Renderer>> {
    let mut map: HashMap<Format, Box<dyn Renderer>> = HashMap::new();
    map.insert(Format::Markdown, Box::new(MarkdownRenderer));
    map.insert(Format::Notebook, Box::new(NotebookRenderer));
    map.insert(Format::Html, Box::new(HtmlRenderer));
    map
}

fn default_parsers() -> HashMap<Format, Box<Parser>> {
    let mut map: HashMap<Format, Box<Parser>> = HashMap::new();
    map.insert(
        Format::Markdown,
        Box::new(Parser {
            preprocessors: vec![
                Box::new(ShortCodeProcessor::new("/templates/shortcodes/**/*", "md").unwrap()),
                Box::new(KaTeXPreprocessor),
            ],
            event_processors: vec![Box::new(CodeSplit)],
        }),
    );
    map.insert(
        Format::Notebook,
        Box::new(Parser {
            preprocessors: vec![
                Box::new(ShortCodeProcessor::new("/templates/shortcodes/**/*", "md").unwrap()),
                Box::new(KaTeXPreprocessor),
            ],
            event_processors: vec![Box::new(CodeSplit)],
        }),
    );

    map.insert(
        Format::Html,
        Box::new(Parser {
            preprocessors: vec![
                Box::new(ShortCodeProcessor::new("/templates/shortcodes/**/*", "md").unwrap()),
                Box::new(KaTeXPreprocessor),
            ],
            event_processors: vec![Box::new(CodeSplit)],
        }),
    );

    map
}
