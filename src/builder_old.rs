use crate::cfg::Format;
use crate::config::{Chapter, Config, DocFrontMatter, Document, Section};
use crate::extensions::{CodeSplit, CodeSplitFactory, Extension, ExtensionFactory};
use crate::notebook::Notebook;
use crate::notebook_writer::render_notebook;
use crate::parsers::split::Rule;
use crate::parsers::split_types::CodeTaskDefinition;
use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use pulldown_cmark::HeadingLevel::H1;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tera::Tera;
use thiserror::Error;
use yaml_front_matter::YamlFrontMatter;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html", ".sql"]);
        tera
    };
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Split parser error")]
    SplitParserError(#[from] pest::error::Error<Rule>),
    #[error("Attribute parser error")]
    AttributeParserError,
}

#[derive(Debug)]
pub struct RenderData {
    pub heading: String,
    pub meta: DocFrontMatter,
    pub html: String,
    pub notebook: Notebook,
    pub raw_solution: String,
    pub split_meta: CodeTaskDefinition,
}

pub struct Builder {
    tera: Tera,
    code_split: CodeSplitFactory,
    extensions: Vec<Box<dyn ExtensionFactory>>,
}

impl Builder {
    pub fn new<P: AsRef<Path>>(
        project_path: P,
        extensions: Vec<Box<dyn ExtensionFactory>>,
    ) -> Result<Self> {
        let path_str = project_path
            .as_ref()
            .to_str()
            .ok_or(anyhow!("Invalid path"))?;
        let pattern = path_str.to_string() + "/templates/**/*.tera.html";
        Ok(Builder {
            tera: Tera::new(&pattern)?,
            code_split: CodeSplitFactory {},
            extensions,
        })
    }

    pub fn parse_pd(&mut self, doc: Document) -> Result<RenderData> {
        let options = Options::all();

        let res = match doc.format {
            Format::Notebook => {
                let bf = BufReader::new(File::open(doc.path)?);
                let nb: Notebook = serde_json::from_reader(bf)?;
                let meta = DocFrontMatter {
                    title: Some("None yet".to_string()),
                    doc_type: "exercise".to_string(),
                };
                self.render(meta, nb.into_iter())
            }
            Format::Markdown => {
                let input = fs::read_to_string(doc.path)?;
                let doc: yaml_front_matter::Document<DocFrontMatter> =
                    YamlFrontMatter::parse(&input).unwrap();
                let parser = Parser::new_ext(&doc.content, options);
                self.render(doc.metadata, parser)
            }
        };

        res
    }

    fn render<'i, I>(&mut self, meta: DocFrontMatter, iter: I) -> Result<RenderData>
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

        let mut html_output = String::new();

        let iter: Result<Vec<Event<'i>>> = iter.collect();
        let iter = iter?;

        let mut i_tmp = iter.clone().into_iter();
        let mut heading = "".to_string();
        while let Some(e) = i_tmp.next() {
            if let Event::Start(tag) = e {
                if let Tag::Heading(lvl, _, _) = tag {
                    match lvl {
                        H1 => {
                            if let Some(txt) = i_tmp.next() {
                                if let Event::Text(actual_text) = txt {
                                    heading = actual_text.into_string();
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        let nb = render_notebook(iter.clone().into_iter())?;

        html::push_html(&mut html_output, iter.into_iter());
        Ok(RenderData {
            heading,
            meta,
            html: html_output,
            notebook: nb,
            raw_solution: code_ext.solution_string,
            split_meta: code_ext.source_def,
        })
    }

    pub fn render_section(
        &self,
        config: &Config,
        section: &Section,
        chapter: &Chapter,
        render_data: &RenderData,
    ) -> Result<String> {
        let mut context = tera::Context::new();
        context.insert("config", config);
        context.insert("current_section", section);
        context.insert("current_chapter", &chapter);
        context.insert("html", &render_data.html);
        context.insert("title", "Test");
        context.insert("meta", &render_data.meta);
        self.tera
            .render("section.tera.html", &context)
            .context("Render error")
    }
}
