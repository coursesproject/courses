use crate::cfg::{DocumentSpec, Format};
use crate::document::{ConfigureIterator, DocPos, Document, IteratorConfig, PreprocessError};
use crate::extensions::shortcode_extender::ShortCodeProcessError;
use crate::extensions::{CodeSplit, Extension, Preprocessor};
use crate::notebook::Notebook;
use crate::notebook_writer::{render_markdown, render_notebook};
use pulldown_cmark::HeadingLevel::H1;
use pulldown_cmark::{html, CowStr, Event, Tag};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;
use yaml_front_matter::YamlFrontMatter;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FrontMatter {
    pub title: Option<String>,
    #[serde(rename = "type", default = "default_title")]
    pub doc_type: String,
    #[serde(default = "default_true")]
    pub code_split: bool,
    #[serde(default = "default_true")]
    pub notebook_output: bool,
    #[serde(default)]
    pub layout: LayoutSettings,

    #[serde(default)]
    pub output: OutputSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputSpec {
    #[serde(default = "default_true")]
    pub web: bool,
    #[serde(default = "default_true")]
    pub source: bool,
}

impl Default for OutputSpec {
    fn default() -> Self {
        OutputSpec {
            web: true,
            source: true,
        }
    }
}

// #[derive(Clone, Debug, Default, Serialize, Deserialize)]
// pub struct SplitSettings {
//     bool_active: bool,
//
// }

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LayoutSettings {
    pub hide_sidebar: bool,
}

fn default_title() -> String {
    "text".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default)]
pub struct DocumentParsed {
    pub(crate) title: String,
    pub(crate) frontmatter: FrontMatter,
    pub(crate) html: String,
    pub(crate) notebook: Notebook,
    pub(crate) md: String,
}

pub struct DocParser {
    project_path: PathBuf,
    html_preprocessors: Vec<Box<dyn Preprocessor>>,
    md_preprocessors: Vec<Box<dyn Preprocessor>>,
}

#[allow(unused)]
struct HeadingNode {
    id: String,
    children: Vec<HeadingNode>,
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
    KaTeX(#[from] katex::Error),

    #[error(transparent)]
    Std(#[from] Box<dyn std::error::Error>),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl DocParser {
    pub fn new<P: AsRef<Path>>(
        project_path: P,
        html_preprocessors: Vec<Box<dyn Preprocessor>>,
        md_preprocessors: Vec<Box<dyn Preprocessor>>,
    ) -> Result<Self, tera::Error> {
        Ok(DocParser {
            project_path: project_path.as_ref().to_path_buf(),
            html_preprocessors,
            md_preprocessors,
        })
    }

    pub fn parse(&mut self, doc: &DocumentSpec<()>) -> Result<DocumentParsed, ParserError> {
        let content_path = self.project_path.join("content").join(&doc.path);
        let res: Result<DocumentParsed, ParserError> = match doc.format {
            Format::Notebook => {
                let mut buf = String::new();
                File::open(&content_path)?.read_to_string(&mut buf)?;
                // let bf = BufReader::new(File::open(&content_path)?);
                let nb: Notebook = serde_json::from_str(&buf)?;
                let meta = nb.get_front_matter()?;
                self.process(Document::from(nb), meta)
            }
            Format::Markdown => {
                let input = fs::read_to_string(&content_path)?;
                let yml: yaml_front_matter::Document<FrontMatter> =
                    YamlFrontMatter::parse(&input).unwrap(); // TODO: HELP!
                                                             // let parser = Parser::new_ext(&yml.content, options);
                self.process(Document::from(yml.content.clone()), yml.metadata)
            }
        };

        res
    }

    fn process_single<'a>(
        &'a self,
        config: IteratorConfig,
        doc: &'a Document,
        meta: FrontMatter,
    ) -> Result<(CodeSplit, Vec<(Event, DocPos)>), crate::extensions::Error> {
        let mut code_ext = CodeSplit::new(meta);
        let iter = doc.configure_iterator(config);

        // let iter = iter.map(|v| code_ext.each(v));
        // let v: Vec<(Event, DocPos)> =
        //     iter.collect::<Result<Vec<(Event, DocPos)>, crate::extensions::Error>>()?;

        let v = code_ext.process(iter)?;

        // let mut hs: HashMap<String, String> = HashMap::new();
        //
        // let iter = v.into_iter().map(|(e, pos)| (match e {
        //     Event::Text(txt) => {
        //         let ts = txt.into_string();
        //         if &ts == "\\" {
        //             Event::Text(CowStr::Boxed("\\\\".to_string().into_boxed_str()))
        //         } else {
        //             Event::Text(CowStr::Boxed(ts.into_boxed_str()))
        //         }
        //     },
        //     Event::Start(tag) => {
        //         match tag {
        //             Tag::Heading(lvl, attr, cls) => {
        //                 hs.insert()
        //             },
        //             t => t
        //         }
        //     },
        //     e => e
        // }, pos));

        let iter = v.into_iter().map(|(e, pos)| {
            (
                if let Event::Text(txt) = e {
                    let ts = txt.into_string();
                    if &ts == "\\" {
                        Event::Text(CowStr::Boxed("\\\\".to_string().into_boxed_str()))
                    } else {
                        Event::Text(CowStr::Boxed(ts.into_boxed_str()))
                    }
                } else {
                    e
                },
                pos,
            )
        });

        let v: Vec<(Event, DocPos)> = iter.collect();

        Ok((code_ext, v))
    }

    fn process(
        &mut self,
        content: Document,
        meta: FrontMatter,
    ) -> Result<DocumentParsed, ParserError> {
        let content_html = self
            .html_preprocessors
            .iter()
            .fold(Ok(content.clone()), |content, preprocessor| {
                content.and_then(|c| c.preprocess(preprocessor.as_ref()))
            })?;
        let content_md = self
            .md_preprocessors
            .iter()
            .fold(Ok(content), |content, preprocessor| {
                content.and_then(|c| c.preprocess(preprocessor.as_ref()))
            })?;

        // let mut content_html = content.preprocess(&processor_html)?;
        // if self.katex_output {
        //     content_html = content_html.preprocess(&processor_html_katex)?;
        // }
        //
        // let content_md = content.preprocess(&processor_export)?;

        let (_code_html, vec_html) = self.process_single(
            IteratorConfig {
                include_output: meta.notebook_output,
                include_solutions: false,
            },
            &content_html,
            meta.clone(),
        )?;
        let (_code_md, vec_md) =
            self.process_single(IteratorConfig::default(), &content_md, meta.clone())?;

        let heading = Self::find_header(&vec_html.clone());

        let nb = render_notebook(vec_md.clone().into_iter())?;
        let md = render_markdown(vec_md.into_iter())?;

        let mut html_output = String::new();
        html::push_html(
            &mut html_output,
            vec_html.into_iter().into_iter().map(|(event, _)| event),
        );

        // html_output = ShortCodeProcessor::new(&self.tera).process(&html_output);

        Ok(DocumentParsed {
            title: heading,
            html: html_output,
            md,
            notebook: nb,
            frontmatter: meta,
        })
    }

    fn find_header(iter: &[(Event, DocPos)]) -> String {
        let mut i_tmp = iter.iter();
        let mut heading = "".to_string();
        while let Some((e, _)) = i_tmp.next() {
            if let Event::Start(Tag::Heading(H1, _, _)) = e {
                if let Some((Event::Text(actual_text), _)) = i_tmp.next() {
                    heading = actual_text.trim().to_string();
                    break;
                }
            }
        }
        heading
    }
}
