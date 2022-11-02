use crate::cfg::{section_id, Config, Chapter, Document, Part, Transform, TransformParents, ConfigItem};
use crate::extensions::ExtensionFactory;
use crate::parser::{DocParser, DocumentParsed, FrontMatter};
use crate::render::HtmlRenderer;
use anyhow::{anyhow, Context, Error};
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use indicatif::ProgressBar;


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentConfig {
    id: String,
    header: String,
    #[serde(flatten)]
    frontmatter: FrontMatter,
}

pub struct Pipeline {
    project_path: PathBuf,
    parser: DocParser,
    renderer: HtmlRenderer,
}

impl Pipeline {
    pub fn new<P: AsRef<Path>>(project_path: P) -> anyhow::Result<Self> {
        Ok(Pipeline {
            project_path: project_path.as_ref().to_path_buf(),
            parser: DocParser::new(project_path.as_ref(), vec![])?,
            renderer: HtmlRenderer::new(project_path.as_ref())?,
        })
    }

    fn parse(project_path: PathBuf, doc: &Document<()>) -> anyhow::Result<DocumentParsed> {
        let mut parser = DocParser::new(project_path, vec![])?;
        parser.parse(doc)
    }

    pub fn build_file<P: AsRef<Path>>(
        &mut self,
        path: P,
        config: &Config<()>,
        build_config: &Config<DocumentConfig>,
    ) -> anyhow::Result<()> {

        // let doc_base = RelativePathBuf::from_path(&path)?;
        // let doc_path = doc_base
        //     .strip_prefix(RelativePathBuf::from_path(&self.project_path)?)
        //     .unwrap();


        let doc_path = path
            .as_ref()
            .strip_prefix(self.project_path.as_path().join("content"))?;
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

        let parsed = Pipeline::parse(self.project_path.clone(), &doc)?;
        let build_path = self.project_path.join("build");

        let mut doc_relative_dir = doc_path.to_path_buf();
        doc_relative_dir.pop();

        let build_dir = build_path.join(doc_relative_dir);
        let html = self.renderer.render_document(&parsed, build_config)?;
        // let mut section_build_dir = build_path.join(part.id.clone()).join(chapter.id.clone());
        let section_build_path = build_dir.join(format!("{}.html", doc.id));

        fs::create_dir_all(build_dir)?;
        fs::write(section_build_path, html).unwrap();
        println!("🔔 Document {} changed, re-rendered output", doc.id);

        Ok(())
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


        let bar = ProgressBar::new(len);
        let parsed: Config<DocumentParsed> = config.into_iter().map(|item| item.map_doc(|doc| {
            bar.inc(1);
            Pipeline::parse(self.project_path.clone(), &doc)
        })).collect::<anyhow::Result<Config<DocumentParsed>>>()?;
        bar.finish();
        // Work on how to create build configuration
        println!("[3/4] 🌵 Generating build configuration...");
        let build_config: Config<DocumentConfig> = parsed.clone().into_iter().map(|item| item.map_doc(|doc| {
            let c = doc.content;
            Ok(DocumentConfig {
                id: doc.id.clone(),
                header: c.title.clone(),
                frontmatter: c.frontmatter.clone(),
            })
        })).collect::<anyhow::Result<Config<DocumentConfig>>>()?;

        let build_config: Config<DocumentConfig> = parsed.clone().into_iter().map(|item| item.map_doc(|doc| {
            let c = doc.content;
            Ok(DocumentConfig {
                id: doc.id.clone(),
                header: c.title.clone(),
                frontmatter: c.frontmatter.clone(),
            })
            // .map_err(|e| {
            //     println!("error: {:?}", e);
            //     e
            // })
            // .ok()
        })).collect::<anyhow::Result<Config<DocumentConfig>>>()?;

        let build_path = self.project_path.join("build");

        println!("[X/4] Writing notebooks...");
        let res: Vec<()> = parsed.clone().into_iter().map(|item|  {
            let mut notebook_build_dir = build_path.as_path().join("source").join(&item.doc.path);
            notebook_build_dir.pop(); // Pop filename
            let notebook_build_path = notebook_build_dir.join(format!("{}.ipynb", item.doc.id));

            fs::create_dir_all(notebook_build_dir)?;
            let f = File::create(notebook_build_path)?;
            let writer = BufWriter::new(f);
            serde_json::to_writer(writer, &item.doc.content.notebook)?;
            Ok(())
        }).collect::<anyhow::Result<Vec<()>>>()?;


        println!("[4/4] 🌼 Rendering output...");
        let res: Vec<()> = parsed.clone().into_iter().map(|item| {
            let output = self.renderer.render_document(&item.doc.content, &build_config)?;

            let mut html_build_dir = build_path.as_path().join("web").join(&item.doc.path);
            html_build_dir.pop(); // Pop filename
            let section_build_path = html_build_dir.join(format!("{}.html", item.doc.id));

            fs::create_dir_all(html_build_dir)?;
            fs::write(section_build_path, output).unwrap();

            Ok(())
        }).collect::<anyhow::Result<Vec<()>>>()?;


        Ok(build_config)
    }
}


