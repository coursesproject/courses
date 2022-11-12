use crate::cfg::{DocumentSpec, Format};
use crate::document::{ConfigureIterator, DocPos, Document, IteratorConfig, PreprocessError};
use crate::extensions::shortcode_extender::ShortCodeProcessor;
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
use std::io::BufReader;
use std::ops::Range;
use std::path::{Path, PathBuf};
use tera::Tera;
use thiserror::Error;
use yaml_front_matter::YamlFrontMatter;

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
    tera_html: Tera,
    tera_md: Tera,
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
    ExtensionError(#[from] crate::extensions::Error)
}

impl DocParser {
    pub fn new<P: AsRef<Path>>(
        project_path: P,
        extensions: Vec<Box<dyn ExtensionFactory>>,
    ) -> Result<Self, tera::Error> {
        let pattern_html = project_path.as_ref().to_str().unwrap().to_string()
            + &format!("/templates/shortcodes/html/**/*.tera.html");
        let pattern_md = project_path.as_ref().to_str().unwrap().to_string()
            + &format!("/templates/shortcodes/md/**/*.tera.md");

        Ok(DocParser {
            project_path: project_path.as_ref().to_path_buf(),
            code_split: CodeSplitFactory {},
            extensions,
            tera_html: Tera::new(&pattern_html)?,
            tera_md: Tera::new(&pattern_md)?,
        })
    }

    pub fn parse(&mut self, doc: &DocumentSpec<()>) -> Result<DocumentParsed, ParserError> {
        let options = Options::all();
        let processor = ShortCodeProcessor::new(&self.tera_md);
        let content_path = self.project_path.join("content").join(&doc.path);
        let res: Result<DocumentParsed, ParserError> = match doc.format {
            Format::Notebook => {
                let bf = BufReader::new(File::open(&content_path)?);
                let nb: Notebook = serde_json::from_reader(bf)?;
                let meta = nb.get_front_matter()?;
                self.process(
                    doc,
                    Document::from(nb.clone()).preprocess(&processor)?,
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
                    Document::from(yml.content.clone()).preprocess(&processor)?,
                    yml.metadata,

                )
            }
        };

        res
    }

    fn process(
        &mut self,
        doc: &DocumentSpec<()>,
        content: Document,
        meta: FrontMatter,
    ) -> Result<DocumentParsed, ParserError>
    {
        let exts: Vec<Box<dyn Extension>> = self.extensions.iter().map(|e| e.build()).collect();

        let iter = content.configure_iterator(IteratorConfig::default().include_output());

        let iter = iter.map(|e| Ok(e));
        let iter = exts.into_iter().fold(
            Box::new(iter) as Box<dyn Iterator<Item = Result<(Event, DocPos), crate::extensions::Error>>>,
            |it, mut ext| Box::new(it.map(move |e| e.and_then(|e| ext.each(e)))),
        );

        let mut code_ext = CodeSplit::default();
        let iter = iter.map(|v| code_ext.each(v?));

        let iter: Result<Vec<(Event, DocPos)>, crate::extensions::Error> = iter.collect();
        let iter = iter?;

        let heading = Self::find_header(&iter);
        // let iter = ShortCodeExtender::from_iter(iter.into_iter(), &self.tera)?;

        let nb = render_notebook(
            content.configure_iterator(IteratorConfig::default().include_solutions()),
        )?;
        let md = render_markdown(content.configure_iterator(IteratorConfig::default()))?;
        let mut html_output = String::new();
        // let new_iter = ShortCodeExtender::new(&self.tera, iter.into_iter());

        html::push_html(&mut html_output, iter.into_iter().map(|(event, _)| event));

        // html_output = ShortCodeProcessor::new(&self.tera).process(&html_output);

        Ok(DocumentParsed {
            title: heading,
            html: html_output,
            md,
            notebook: nb,
            doc_content: content,
            raw_solution: code_ext.solution_string,
            split_meta: code_ext.source_def,
            frontmatter: meta,
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
