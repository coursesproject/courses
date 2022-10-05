use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use crate::config::{Document, Format};
use crate::Config;
use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use pandoc::{InputFormat, MarkdownExtension, OutputKind, Pandoc, PandocOutput};
use pulldown_cmark::{Event, html, Options, Parser, Tag};
use tera::Tera;
use courses::notebook::Notebook;

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


pub struct Builder {
    tera: Tera
}

impl Builder {
    pub fn new<P: AsRef<Path>>(project_path: P) -> Result<Self> {
        let path_str = project_path.as_ref().to_str().ok_or(anyhow!("Invalid path"))?;
        let pattern = path_str.to_string() + "/templates/**/*.tera.html";
        Ok(Builder {
            tera: Tera::new(&pattern)?
        })
    }

    pub fn parse_pd(doc: Document) -> Result<String> {
        let mut options = Options::empty();

        let html_output = match doc.format {
            Format::Notebook => {
                let bf = BufReader::new(File::open(doc.path)?);
                let nb: Notebook = serde_json::from_reader(bf)?;
                Builder::render(nb.into_iter())
            }
            Format::Markdown => {
                let input = fs::read_to_string(doc.path)?;
                let parser = Parser::new_ext(&input, options);
                Builder::render(parser)
            }
        };

        Ok(html_output)
    }

    fn render<'a, I>(iter: I) -> String where I: Iterator<Item=Event<'a>> {
        let mut html_output = String::new();

        let iter = iter.map(|e| {
            match e {
                Event::Start(tag) => {
                    Event::Start(match tag {
                        Tag::Heading(lvl, id, cls) => Tag::Heading(lvl, Some("myid"), cls),
                        _ => tag
                    })
                },
                _ => e
            }
        });
        html::push_html(&mut html_output, iter);
        html_output
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

