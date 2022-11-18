use crate::cfg::{DocumentSpec, Format};
use crate::document::{ConfigureIterator, DocPos, Document, IteratorConfig, PreprocessError};
use crate::extensions::shortcode_extender::{ShortCodeProcessError, ShortCodeProcessor};
use crate::extensions::{CodeSplit, CodeSplitFactory, Extension, ExtensionFactory};
use crate::notebook::Notebook;
use crate::notebook_writer::{render_markdown, render_notebook};
use crate::parsers::split_types::CodeTaskDefinition;
use anyhow::Context;
use pulldown_cmark::HeadingLevel::H1;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::ops::Range;
use std::path::{Path, PathBuf};
use katex::{Opts, OptsBuilder};
use tera::Tera;
use thiserror::Error;
use yaml_front_matter::YamlFrontMatter;
use crate::extensions::katex::{KaTeXPreprocessor, KaTeXPreprocessorError};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FrontMatter {
    pub title: Option<String>,
    #[serde(rename = "type", default = "default_doc_type")]
    pub doc_type: String,
}

fn default_doc_type() -> String {
    "text".to_string()
}

#[derive(Debug, Clone, Default)]
pub struct DocumentParsed {
    pub(crate) title: String,
    pub(crate) frontmatter: FrontMatter,
    pub(crate) doc_content: Document,
    pub(crate) doc_export: Document,
    pub(crate) html: String,
    pub(crate) notebook: Notebook,
    pub(crate) md: String,
    pub(crate) raw_solution: String,
    pub(crate) split_meta: CodeTaskDefinition,
}

pub struct DocParser {
    project_path: PathBuf,
    code_split: CodeSplitFactory,
    extensions: Vec<Box<dyn ExtensionFactory>>,
    katex_opts: Opts,
    katex_output: bool,
    tera: Tera,
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("IO Error: ")]
    IoError(#[from] std::io::Error),

    #[error("Error in template")]
    TemplateError(#[from] tera::Error),

    #[error("JSON Error: ")]
    JSONError(#[from] serde_json::error::Error),

    #[error("Error parsing frontmatter: ")]
    FrontMatter(#[from] serde_yaml::Error),

    #[error(transparent)]
    Preprocess(#[from] PreprocessError),

    #[error(transparent)]
    ExtensionError(#[from] crate::extensions::Error),

    #[error(transparent)]
    ShortCode(#[from] ShortCodeProcessError),

    #[error(transparent)]
    KaTeX(#[from] katex::Error)
}

impl DocParser {
    pub fn new<P: AsRef<Path>>(
        project_path: P,
        extensions: Vec<Box<dyn ExtensionFactory>>,
        katex_opts: Opts,
        katex_output: bool,
    ) -> Result<Self, tera::Error> {
        let pattern = project_path.as_ref().to_str().unwrap().to_string()
            + &format!("/templates/shortcodes/**/*");

        Ok(DocParser {
            project_path: project_path.as_ref().to_path_buf(),
            code_split: CodeSplitFactory {},
            extensions,
            tera: Tera::new(&pattern)?,
            katex_opts,
            katex_output,
        })
    }

    pub fn parse(&mut self, doc: &DocumentSpec<()>) -> Result<DocumentParsed, ParserError> {
        let options = Options::all();

        let content_path = self.project_path.join("content").join(&doc.path);
        let res: Result<DocumentParsed, ParserError> = match doc.format {
            Format::Notebook => {
                let mut buf = String::new();
                File::open(&content_path)?.read_to_string(&mut buf)?;
                // let bf = BufReader::new(File::open(&content_path)?);
                let nb: Notebook = serde_json::from_str(&buf)?;
                let meta = nb.get_front_matter()?;
                self.process(
                    doc,
                    Document::from(nb.clone()),
                    meta,
                )
            }
            Format::Markdown => {
                let input = fs::read_to_string(&content_path)?;
                let yml: yaml_front_matter::Document<FrontMatter> =
                    YamlFrontMatter::parse(&input).unwrap(); // TODO: HELP!
                let parser = Parser::new_ext(&yml.content, options);
                self.process(
                    doc,
                    Document::from(yml.content.clone()),
                    yml.metadata,
                )
            }
        };

        res
    }

    fn process_single<'a>(&'a self, config: IteratorConfig, doc: &'a Document) -> Result<(CodeSplit, Vec<(Event, DocPos)>), crate::extensions::Error> {
        let mut code_ext = CodeSplit::default();
        let iter = doc.configure_iterator(config);
        let iter = iter.map(|v| code_ext.each(v));
        let v: Vec<(Event, DocPos)> = iter.collect::<Result<Vec<(Event, DocPos)>, crate::extensions::Error>>()?;
        Ok((code_ext, v))
    }

    fn process(
        &mut self,
        doc: &DocumentSpec<()>,
        content: Document,
        meta: FrontMatter,
    ) -> Result<DocumentParsed, ParserError>
    {
        let processor_html_katex = KaTeXPreprocessor::new(self.katex_opts.clone());
        let processor_html = ShortCodeProcessor::new(&self.tera, "html".to_string());
        let processor_export = ShortCodeProcessor::new(&self.tera, "md".to_string());

        let mut content_html = content.preprocess(&processor_html)?;
        if self.katex_output {
            content_html = content_html.preprocess(&processor_html_katex)?;
        }

        let content_md = content.preprocess(&processor_export)?;

        let (code_html, vec_html) = self.process_single(IteratorConfig::default().include_output(), &content_html)?;
        let (code_md, vec_md) = self.process_single(IteratorConfig::default(), &content_md)?;

        let heading = Self::find_header(&vec_html.clone());


        let nb = render_notebook(vec_md.clone().into_iter())?;
        let md = render_markdown(vec_md.into_iter())?;

        let mut html_output = String::new();
        html::push_html(&mut html_output, vec_html.into_iter().into_iter().map(|(event, _)| event));

        // html_output = ShortCodeProcessor::new(&self.tera).process(&html_output);

        Ok(DocumentParsed {
            title: heading,
            html: html_output,
            md,
            notebook: nb,
            doc_content: content_html,
            raw_solution: code_html.solution_string,
            split_meta: code_html.source_def,
            frontmatter: meta,
            doc_export: content_md
        })
    }

    fn find_header(iter: &Vec<(Event, DocPos)>) -> String {
        let mut i_tmp = iter.clone().into_iter();
        let mut heading = "".to_string();
        while let Some((e, _)) = i_tmp.next() {
            if let Event::Start(tag) = e {
                if let Tag::Heading(lvl, _, _) = tag {
                    match lvl {
                        H1 => {
                            if let Some((txt, _)) = i_tmp.next() {
                                if let Event::Text(actual_text) = txt {
                                    heading = actual_text.trim().to_string();
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        heading
    }
}
