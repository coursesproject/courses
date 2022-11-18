use crate::cfg::{section_id, Config, ConfigItem, DocumentSpec, Part, ProjectConfig};
use crate::parser::{DocParser, DocumentParsed, FrontMatter, ParserError};
use crate::render::{HtmlRenderError, HtmlRenderer};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::render::HtmlRenderError::TemplateError;
use indicatif::ProgressBar;
use katex::OptsBuilder;
use termion::{color, style};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentConfig {
    id: String,
    header: String,
    #[serde(flatten)]
    frontmatter: FrontMatter,
}

pub struct Pipeline {
    project_path: PathBuf,
    project_config: ProjectConfig,
    renderer: HtmlRenderer,
}

impl Pipeline {
    pub fn new<P: AsRef<Path>>(project_path: P) -> anyhow::Result<Self> {
        let config_path = project_path.as_ref().join("config.yml");
        let config_reader = BufReader::new(File::open(config_path)?);

        let config: ProjectConfig = serde_yaml::from_reader(config_reader)?;

        Ok(Pipeline {
            project_path: project_path.as_ref().to_path_buf(),
            renderer: HtmlRenderer::new(project_path.as_ref())?,
            project_config: config,
        })
    }

    fn parse(&self, doc: &DocumentSpec<()>) -> Result<DocumentParsed, ParserError> {
        let opts = OptsBuilder::default().build().unwrap();
        let mut parser = DocParser::new(self.project_path.clone(), vec![], opts, self.project_config.katex_output)?;
        parser.parse(doc)
    }

    pub fn build_file<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: &Config<()>,
        build_config: &Config<DocumentConfig>,
    ) {
        // let doc_base = RelativePathBuf::from_path(&path)?;
        // let doc_path = doc_base
        //     .strip_prefix(RelativePathBuf::from_path(&self.project_path)?)
        //     .unwrap();

        let doc_path = path
            .as_ref()
            .strip_prefix(self.project_path.as_path().join("content"))
            .unwrap(); // TODO: Error handling;
        let mut doc_iter = doc_path.iter();
        let el = doc_iter.next().unwrap().to_str().unwrap();

        let doc = if el.contains(".") {
            &config.index
        } else {
            let first_elem = doc_iter.next().unwrap().to_str().unwrap();

            // let file_name = doc_iter.next().unwrap().to_str().unwrap();
            let file_id = section_id(path.as_ref()).unwrap();

            let part: &Part<()> = &config
                .content
                .iter()
                .filter(|e| &e.id == el)
                .next()
                .unwrap();
            let elem = part.chapters.iter().filter(|c| &c.id == first_elem).next();

            match elem {
                None => &part.index,
                Some(c) => {
                    let doc = c.documents.iter().filter(|d| d.id == file_id).next();
                    match doc {
                        None => &c.index,
                        Some(d) => &d,
                    }
                }
            }
        };

        let parsed = self.parse(doc); // TODO: Error message
        match parsed {
            Ok(parsed) => {
                let parsed_doc = DocumentSpec {
                    id: doc.id.clone(),
                    format: doc.format.clone(),
                    path: doc.path.clone(),
                    content: Arc::new(parsed),
                };

                let basebuild_path = self.project_path.join("build");
                // let build_path = self.project_path.join("build").join("web");
                //
                // let mut doc_relative_dir = doc_path.to_path_buf();
                // doc_relative_dir.pop();
                //
                // let build_dir = build_path.join(doc_relative_dir);
                // let html = self.renderer.render_document(&parsed, build_config)?;
                // // let mut section_build_dir = build_path.join(part.id.clone()).join(chapter.id.clone());
                // let section_build_path = build_dir.join(format!("{}.html", doc.id));
                //
                // fs::create_dir_all(build_dir)?;
                // fs::write(section_build_path, html).unwrap();

                self.write_html(&parsed_doc, build_config, &basebuild_path)
                    .unwrap(); // TODO: Error handling
                self.write_notebook(&parsed_doc, &basebuild_path).unwrap(); // TODO: Error handling

                println!("🔔 Document {} changed, re-rendered output", doc.id);
            }
            Err(e) => {
                println!("Error {}", e);
            }
        }

    }

    pub fn build_everything(
        &mut self,
        config: Config<()>,
    ) -> anyhow::Result<Config<DocumentConfig>> {
        let mut len: u64 = 0;
        for p in &config.content {
            len += 1;
            for c in &p.chapters {
                len += c.documents.len() as u64 + 1;
            }
        }

        println!("[2/4] 📖 Parsing source documents...");

        // let bar = ProgressBar::new(len);

        let parsed: Config<DocumentParsed> = config
            .into_iter()
            .map(|item| {
                let res = item.map_doc(|doc| self.parse(&doc));
                let res = match res {
                    Ok(i) => Some(i),
                    Err(e) => {
                        let mut ei: &dyn Error = &e;
                        // bar.println(format!("{}{}error: {}{}{}\n", style::Bold, color::Fg(color::Red), style::Reset, color::Fg(color::Reset), ei));
                        println!("{}{}error: {}{}{}", style::Bold, color::Fg(color::Red), style::Reset, color::Fg(color::Reset), ei);
                        // while let Some(inner) = ei.source() {
                        //     // bar.println(format!("Caused by: {}\n", inner));
                        //     println!("{}cause: {}{}", style::Bold, style::Reset, inner);
                        //     ei = inner;
                        // }

                        None
                    }
                };
                // bar.inc(1);
                res
            })
            .filter_map(|res| res)
            .collect::<Config<DocumentParsed>>();
        // bar.finish();

        // Work on how to create build configuration
        println!("[3/4] 🌵 Generating build configuration...");
        let build_config: Config<DocumentConfig> = parsed
            .clone()
            .into_iter()
            .map(|item| {
                item.map_doc(|doc| {
                    let c = doc.content;
                    Ok(DocumentConfig {
                        id: doc.id.clone(),
                        header: c.title.clone(),
                        frontmatter: c.frontmatter.clone(),
                    })
                })
            })
            .collect::<anyhow::Result<Config<DocumentConfig>>>()?;

        let build_path = self.project_path.join("build");

        println!("[X/4] Writing notebooks...");
        let notebook_errors: Vec<()> = parsed
            .clone()
            .into_iter()
            .map(|item| self.write_notebook(&item.doc, &build_path))
            .collect::<anyhow::Result<Vec<()>>>()?;

        println!("[4/4] 🌼 Rendering output...");
        let html_results_filtered: Vec<ConfigItem<DocumentParsed>> = parsed
            .clone()
            .into_iter()
            .map(|item| {
                self.write_html(&item.doc, &build_config, &build_path)
                    .map(|_| item)
            })
            .filter_map(|result| match result {
                Ok(i) => Some(i),
                Err(e) => {
                    match e {
                        HtmlRenderError::TemplateError(e, title) => {
                            println!(
                                "{}[Error] {}Could not render '{}' due to:",
                                color::Fg(color::Red),
                                color::Fg(color::Black),
                                title
                            );
                            println!("\t{}", e);
                            if let Some(source) = e.source() {
                                println!("\t\tCaused by: {}", source);
                            }
                        }
                        HtmlRenderError::Other(e, title) => {
                            println!("[Error] Could not render '{}' due to:", title);
                            println!("[Error] {}", e);
                        }
                    }
                    None
                }
            })
            .collect();

        let md_errors: Vec<ConfigItem<DocumentParsed>> = html_results_filtered
            // .clone()
            .into_iter()
            .map(|item| self.write_markdown(&item.doc, &build_path).map(|_| item))
            .filter_map(|result| match result {
                Ok(i) => Some(i),
                Err(e) => {
                    println!("[Error] Markdown render error: {}", e);
                    None
                }
            })
            .collect();

        Ok(build_config)
    }

    fn write_notebook<P: AsRef<Path>>(
        &self,
        doc: &DocumentSpec<DocumentParsed>,
        build_path: P,
    ) -> anyhow::Result<()> {
        let mut notebook_build_dir = build_path.as_ref().join("source").join(&doc.path);
        notebook_build_dir.pop(); // Pop filename
        let notebook_build_path = notebook_build_dir.join(format!("{}.ipynb", doc.id));

        fs::create_dir_all(notebook_build_dir)?;
        let f = File::create(notebook_build_path)?;
        let writer = BufWriter::new(f);
        serde_json::to_writer(writer, &doc.content.notebook)?;
        Ok(())
    }

    fn write_markdown<P: AsRef<Path>>(
        &self,
        doc: &DocumentSpec<DocumentParsed>,
        build_path: P,
    ) -> anyhow::Result<()> {
        let mut md_build_dir = build_path.as_ref().join("md").join(&doc.path);
        md_build_dir.pop(); // Pop filename
        let md_build_path = md_build_dir.join(format!("{}.md", doc.id));

        fs::create_dir_all(md_build_dir)?;
        fs::write(md_build_path, &doc.content.md).unwrap();
        Ok(())
    }

    fn write_html<P: AsRef<Path>>(
        &self,
        doc: &DocumentSpec<DocumentParsed>,
        build_config: &Config<DocumentConfig>,
        build_path: P,
    ) -> Result<(), HtmlRenderError> {
        let output = self
            .renderer
            .render_document(&doc.content, &build_config)
            .map_err(|e| TemplateError(e, doc.path.to_str().unwrap().to_string()))?;

        let mut html_build_dir = build_path.as_ref().join("web").join(&doc.path);
        html_build_dir.pop(); // Pop filename
        let section_build_path = html_build_dir.join(format!("{}.html", doc.id));

        fs::create_dir_all(html_build_dir)
            .context("Could not create directory")
            .map_err(|e| HtmlRenderError::Other(e, doc.path.to_str().unwrap().to_string()))?;
        fs::write(section_build_path, output).unwrap();

        Ok(())
    }
}
