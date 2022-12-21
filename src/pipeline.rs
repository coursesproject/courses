use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use tera::Tera;

use cdoc::config::OutputFormat;
use cdoc::document::Document;
use cdoc::processors::PreprocessorContext;
use cdoc::renderers::RenderResult;

use crate::generators::html::HtmlGenerator;
use crate::generators::info::InfoGenerator;
use crate::generators::notebook::CodeOutputGenerator;
use crate::generators::{Generator, GeneratorContext, MoveContext, Mover};
use crate::project::config::ProjectConfig;
use crate::project::{section_id, ItemDescriptor, Part, Project, ProjectItem};

pub struct Pipeline {
    #[allow(unused)]
    mode: String,
    project_path: PathBuf,
    project: Project<()>,
    project_config: ProjectConfig,
    base_tera: Tera,
    shortcode_tera: Tera,
    cached_contexts: HashMap<OutputFormat, GeneratorContext>,
}

impl Pipeline {
    pub fn new<P: AsRef<Path>>(
        project_path: P,
        mode: String,
        config: ProjectConfig,
        project: Project<()>,
    ) -> anyhow::Result<Self> {
        let path_str = project_path
            .as_ref()
            .to_str()
            .ok_or_else(|| anyhow!("Invalid path"))?;
        let pattern = path_str.to_string() + "/templates/**/*.tera.html";
        let base_tera = Tera::new(&pattern).context("Error preparing project templates")?;

        let shortcode_pattern = path_str.to_string() + "/templates/shortcodes/**/*";
        let shortcode_tera =
            Tera::new(&shortcode_pattern).context("Error preparing project templates")?;

        Ok(Pipeline {
            mode,
            project_path: project_path.as_ref().to_path_buf(),
            project,
            project_config: config,
            base_tera,
            shortcode_tera,
            cached_contexts: HashMap::new(),
        })
    }

    fn get_generator(&self, format: OutputFormat) -> Box<dyn Generator> {
        match format {
            OutputFormat::Notebook => Box::new(CodeOutputGenerator),
            OutputFormat::Html => Box::new(HtmlGenerator::new(self.base_tera.clone())),
            OutputFormat::Info => Box::new(InfoGenerator),
        }
    }

    fn get_build_path(&self, format: OutputFormat) -> PathBuf {
        match format {
            OutputFormat::Notebook => self.project_path.join("build").join("notebooks"),
            OutputFormat::Html => self.project_path.join("build").join("html"),
            OutputFormat::Info => self.project_path.join("build"),
        }
    }

    pub fn build_single(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let item = self.doc_from_path(path)?;
        let item2 = item.clone();

        let loaded = item.map_doc(|doc| {
            let path = self.project_path.join("content").join(doc.path);
            let val = fs::read_to_string(path.as_path())
                .context(format!("Error loading document at {}", path.display()))?;
            Ok::<String, anyhow::Error>(val)
        })?;

        for format in &self.project_config.outputs {
            let output = self.process_document(&loaded.doc, *format)?;

            if let Some(output) = output {
                let context = self
                    .cached_contexts
                    .get(format)
                    .ok_or_else(|| anyhow!("Cached context is missing"))?;

                self.get_generator(*format).generate_single(
                    output,
                    item2.clone(),
                    context.clone(),
                )?;
            }
        }

        Ok(())
    }

    fn doc_from_path(&self, path: PathBuf) -> anyhow::Result<ItemDescriptor<()>> {
        let mut part_id = None;
        let mut chapter_id = None;

        let doc_path = path
            .as_path()
            .strip_prefix(self.project_path.as_path().join("content"))?; // TODO: Error handling;
        let mut doc_iter = doc_path.iter();
        let el = doc_iter.next().unwrap().to_str().unwrap();

        let doc = if el.contains('.') {
            &self.project.index
        } else {
            let first_elem = doc_iter.next().unwrap().to_str().unwrap();

            // let file_name = doc_iter.next().unwrap().to_str().unwrap();
            let file_id = section_id(path.as_path()).unwrap();

            let part: &Part<()> = self
                .project
                .content
                .iter()
                .find(|e| e.id == el)
                .expect("Part not found for single document");
            let elem = part.chapters.iter().find(|c| c.id == first_elem);
            part_id = Some(part.id.clone());

            match elem {
                None => &part.index,
                Some(c) => {
                    chapter_id = Some(c.id.clone());
                    let doc = c.documents.iter().find(|d| d.id == file_id);
                    match doc {
                        None => &c.index,
                        Some(d) => d,
                    }
                }
            }
        };

        let item = ItemDescriptor {
            part_id,
            chapter_id,
            part_idx: None,
            chapter_idx: None,
            doc: doc.clone(),
            files: None,
        };

        Ok(item)
    }

    pub fn build_all(&mut self) -> Result<(), anyhow::Error> {
        let build_path = self.project_path.join("build");

        if build_path.exists() {
            fs::remove_dir_all(build_path)?;
        }

        let loaded = self.load_all()?;

        for format in &self.project_config.outputs {
            let output = self.process_all(loaded.clone(), *format)?;
            let context = GeneratorContext {
                root: self.project_path.to_path_buf(),
                project: output,
                config: self.project_config.clone(),
                build_dir: self.get_build_path(*format),
            };
            self.cached_contexts.insert(*format, context.clone());

            self.get_generator(*format)
                .generate(context.clone())
                .with_context(|| format!("Error while generating {}", format))?;

            self.cached_contexts.insert(*format, context);

            if let Some(parser) = self.project_config.parsers.get(format) {
                let move_ctx = MoveContext {
                    project_path: self.project_path.to_path_buf(),
                    build_dir: self.get_build_path(*format),
                    settings: parser.settings.clone(),
                };

                Mover::traverse_dir(self.project_path.join("content").to_path_buf(), &move_ctx)?;
            }
        }
        Ok(())
    }

    fn load_all(&self) -> Result<Project<String>, anyhow::Error> {
        self.project
            .clone()
            .into_iter()
            .map(|item| {
                item.map_doc(|doc| {
                    let path = self.project_path.join("content").join(doc.path);
                    let val = fs::read_to_string(path.as_path())
                        .context(format!("Error loading document {}", path.display()))?;

                    Ok(val)
                })
            })
            .collect::<Result<Project<String>, anyhow::Error>>()
    }

    // fn load_single(&self, )

    fn process_all(
        &self,
        project: Project<String>,
        format: OutputFormat,
    ) -> anyhow::Result<Project<Option<Document<RenderResult>>>> {
        project
            .into_iter()
            .map(|i| {
                i.map_doc(|doc| {
                    self.process_document(&doc, format)
                        .with_context(|| format!("Failed to parse document {}", doc.path.display()))
                })
            })
            .collect()
    }

    fn process_document(
        &self,
        item: &ProjectItem<String>,
        format: OutputFormat,
    ) -> anyhow::Result<Option<Document<RenderResult>>> {
        let doc = item.format.loader().load(&item.content)?;

        if doc.metadata.outputs.contains(&format) {
            let processor_ctx = PreprocessorContext {
                tera: self.shortcode_tera.clone(),
                output_format: format,
            };

            let mut meta = tera::Context::new();
            meta.insert("project", &self.project_config);

            let res = self
                .project_config
                .parsers
                .get(&format)
                .ok_or_else(|| anyhow!("No spec found"))?
                .parse(&doc, &meta, &processor_ctx)?;

            if let Some(renderer) = format.renderer() {
                Ok(Some(renderer.render(&res)))
            } else {
                Ok(None)
            }

            // let mut build_dir = self
            //     .ctx
            //     .build_path
            //     .as_path()
            //     .join(&self.format)
            //     .join(&item.doc.path);
            // build_dir.pop(); // Pop filename so only directory remains
            //
            // let file_path = build_dir.join(format!("{}.{}", item.doc.id, self.format));
            // fs::create_dir_all(build_dir)?;
            // fs::write(file_path, output)?;
        } else {
            Ok(None)
        }
    }
}
