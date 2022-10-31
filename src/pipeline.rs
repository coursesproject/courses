use crate::cfg::{section_id, BuildConfig, Chapter, Document, Part, Transform, TransformParents};
use crate::extensions::ExtensionFactory;
use crate::parser::{DocParser, DocumentParsed, FrontMatter};
use crate::render::HtmlRenderer;
use anyhow::{anyhow, Context, Error};
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        config: &BuildConfig<()>,
        build_config: &BuildConfig<Option<DocumentConfig>>,
    ) -> anyhow::Result<()> {
        println!("Started");
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
        println!("Finished {}", doc.id);

        Ok(())
    }

    pub fn build_everything(
        &mut self,
        config: BuildConfig<()>,
    ) -> anyhow::Result<BuildConfig<Option<DocumentConfig>>> {
        let parsed = config.transform(&|doc| Pipeline::parse(self.project_path.clone(), doc));

        let build_config = parsed.transform(&|doc| {
            doc.content
                .as_ref()
                .map(|c| DocumentConfig {
                    id: doc.id.clone(),
                    header: c.title.clone(),
                    frontmatter: c.frontmatter.clone(),
                })
                .map_err(|e| {
                    println!("error: {:?}", e);
                    e
                })
                .ok()
        });

        let build_path = self.project_path.join("build");
        let res: BuildConfig<anyhow::Result<()>> =
            parsed.transform_parents(&|doc, part, chapter| {
                let r = doc.content.as_ref().unwrap(); //TODO: Fix
                let res = self.renderer.render_document(&r, &build_config);
                match res {
                    Ok(res) => {
                        let mut section_build_dir = build_path.clone();

                        if let Some(pt) = part {
                            section_build_dir = section_build_dir.join(pt.id.clone());
                        }

                        if let Some(ch) = chapter {
                            section_build_dir = section_build_dir.join(ch.id.clone())
                        };

                        let section_build_path = section_build_dir.join(format!("{}.html", doc.id));

                        fs::create_dir_all(section_build_dir)?;
                        fs::write(section_build_path, res).unwrap();
                        println!("Finished {}", doc.id);
                        Ok(())
                    }
                    Err(e) => {
                        panic!("{:?}", e)
                    }
                }
            });
        Ok(build_config)
    }
}
