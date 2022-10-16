use crate::config::{Config, Document, Format};
use crate::extensions::{CodeSplit, CodeSplitFactory, Extension, ExtensionFactory};
use crate::notebook::Notebook;
use crate::split::types::CodeTaskDefinition;
use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use pandoc::{InputFormat, MarkdownExtension, OutputKind, Pandoc, PandocOutput};
use pulldown_cmark::{html, Event, Options, Parser};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tera::Tera;

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

#[derive(Debug)]
pub struct RenderData {
    pub html: String,
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

        let html_output = match doc.format {
            Format::Notebook => {
                let bf = BufReader::new(File::open(doc.path)?);
                let nb: Notebook = serde_json::from_reader(bf)?;
                self.render(nb.into_iter())
            }
            Format::Markdown => {
                let input = fs::read_to_string(doc.path)?;
                let parser = Parser::new_ext(&input, options);
                self.render(parser)
            }
        };

        html_output
    }

    fn render<'i, I>(&mut self, iter: I) -> Result<RenderData>
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
        html::push_html(&mut html_output, iter?.into_iter());
        Ok(RenderData {
            html: html_output,
            raw_solution: code_ext.solution_string,
            split_meta: code_ext.source_def,
        })
    }

    pub fn render_section(&self, config: &Config, content: String) -> Result<String> {
        let mut context = tera::Context::new();
        context.insert("config", config);
        context.insert("html", &content);
        context.insert("title", "Test");
        self.tera
            .render("section.tera.html", &context)
            .context("Render error")
    }
}

pub fn parse(doc: Document) -> Result<String> {
    let mut pandoc = Pandoc::new();

    let extensions = vec![
        MarkdownExtension::FencedCodeBlocks,
        MarkdownExtension::FencedCodeAttributes,
        MarkdownExtension::BracketedSpans,
        MarkdownExtension::FencedDivs,
        MarkdownExtension::TexMathDollars,
        MarkdownExtension::BacktickCodeBlocks,
    ];

    match doc.format {
        Format::Markdown => pandoc
            .add_input(doc.path.as_path())
            .set_input_format(InputFormat::Markdown, extensions),

        Format::Notebook => pandoc
            .add_input(doc.path.as_path())
            .set_input_format(InputFormat::Other("ipynb".to_string()), extensions),
    };

    pandoc.set_output(OutputKind::Pipe);
    let output = pandoc
        .execute()
        .with_context(|| format!("Document: {}", doc.path.display()))?;

    if let PandocOutput::ToBuffer(string) = output {
        Ok(string)
    } else {
        panic!("No buffer");
    }
}
