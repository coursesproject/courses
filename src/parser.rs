use crate::builder_old::{Builder, RenderData};
use crate::cfg::{DocumentSpec, Format};
use crate::extensions::{CodeSplit, CodeSplitFactory, Extension, ExtensionFactory};
use crate::notebook::Notebook;
use crate::notebook_writer::{render_markdown, render_notebook};
use crate::parsers::split_types::CodeTaskDefinition;
use anyhow::anyhow;
use pulldown_cmark::HeadingLevel::H1;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tera::Tera;
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
}

impl DocParser {
    pub fn new<P: AsRef<Path>>(
        project_path: P,
        extensions: Vec<Box<dyn ExtensionFactory>>,
    ) -> anyhow::Result<Self> {
        Ok(DocParser {
            project_path: project_path.as_ref().to_path_buf(),
            code_split: CodeSplitFactory {},
            extensions,
        })
    }

    pub fn parse(&mut self, doc: &DocumentSpec<()>) -> anyhow::Result<DocumentParsed> {
        let options = Options::all();

        let content_path = self.project_path.join("content").join(&doc.path);
        let res = match doc.format {
            Format::Notebook => {
                let bf = BufReader::new(File::open(&content_path)?);
                let nb: Notebook = serde_json::from_reader(bf)?;
                let meta = nb.get_front_matter().unwrap().unwrap_or_default();
                self.process(doc, meta, nb.into_iter())
            }
            Format::Markdown => {
                let input = fs::read_to_string(&content_path)?;
                let yml: yaml_front_matter::Document<FrontMatter> =
                    YamlFrontMatter::parse(&input).unwrap();
                let parser = Parser::new_ext(&yml.content, options);
                self.process(doc, yml.metadata, parser)
            }
        };

        res
    }

    fn process<'i, I>(
        &mut self,
        doc: &DocumentSpec<()>,
        meta: FrontMatter,
        iter: I,
    ) -> anyhow::Result<DocumentParsed>
    where
        I: Iterator<Item = Event<'i>>,
    {
        let exts: Vec<Box<dyn Extension<'i>>> = self.extensions.iter().map(|e| e.build()).collect();

        let iter = iter.map(|e| Ok(e));
        let iter = exts.into_iter().fold(
            Box::new(iter) as Box<dyn Iterator<Item = anyhow::Result<Event<'i>>>>,
            |it, mut ext| Box::new(it.map(move |e| e.and_then(|e| ext.each(e)))),
        );

        let mut code_ext = CodeSplit::default();
        let iter = iter.map(|v| code_ext.each(v?));

        let iter: anyhow::Result<Vec<Event<'i>>> = iter.collect();
        let iter = iter?;

        let heading = Self::find_header(&iter);

        let nb = render_notebook(iter.clone().into_iter())?;
        let md = render_markdown(iter.clone().into_iter())?;
        let mut html_output = String::new();
        html::push_html(&mut html_output, iter.into_iter());

        Ok(DocumentParsed {
            title: heading,
            html: html_output,
            md,
            notebook: nb,
            raw_solution: code_ext.solution_string,
            split_meta: code_ext.source_def,
            frontmatter: meta,
        })
    }

    fn find_header(iter: &Vec<Event>) -> String {
        let mut i_tmp = iter.clone().into_iter();
        let mut heading = "".to_string();
        while let Some(e) = i_tmp.next() {
            if let Event::Start(tag) = e {
                if let Tag::Heading(lvl, _, _) = tag {
                    match lvl {
                        H1 => {
                            if let Some(txt) = i_tmp.next() {
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
