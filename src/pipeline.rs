use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use tera::Tera;

use cdoc::config::OutputFormat;
use cdoc::processors::ProcessorContext;
use cdoc::renderers::RenderResult;

use crate::generators::config::ConfigGenerator;
use crate::generators::html::HtmlGenerator;
use crate::generators::markdown::MarkdownGenerator;
use crate::generators::notebook::CodeOutputGenerator;
use crate::generators::{Generator, GeneratorContext};
use crate::project::config::ProjectConfig;
use crate::project::{section_id, Item, Part, Project, ProjectItem};

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
            OutputFormat::Markdown => Box::new(MarkdownGenerator),
            OutputFormat::Notebook => Box::new(CodeOutputGenerator),
            OutputFormat::Html => Box::new(HtmlGenerator::new(
                self.base_tera.clone(),
                self.project.clone(),
            )),
            OutputFormat::Config => Box::new(ConfigGenerator),
        }
    }

    fn get_build_path(&self, format: OutputFormat) -> PathBuf {
        match format {
            OutputFormat::Markdown => self.project_path.join("build").join("md"),
            OutputFormat::Notebook => self.project_path.join("build").join("notebooks"),
            OutputFormat::Html => self.project_path.join("build").join("html"),
            OutputFormat::Config => self.project_path.join("build").join("config"),
        }
    }

    pub fn build_single(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let item = self.doc_from_path(path)?;
        let item2 = item.clone();

        let loaded = item.map_doc(|doc| {
            let path = self.project_path.join("content").join(&doc.path);
            let val = fs::read_to_string(path.as_path())
                .context(format!("Error loading document at {}", path.display()))?;
            Ok::<String, anyhow::Error>(val)
        })?;

        for format in &self.project_config.outputs {
            let output = self.process_document(&loaded.doc, *format)?;

            if let Some(output) = output {
                self.get_generator(*format).generate_single(
                    output,
                    item2.clone(),
                    self.project_config.clone(),
                    self.get_build_path(*format),
                )?;
            }
        }

        Ok(())
    }

    fn doc_from_path(&self, path: PathBuf) -> anyhow::Result<ProjectItem<()>> {
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

        let item = ProjectItem {
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

            self.get_generator(*format).generate(context)?;
        }
        Ok(())
    }

    fn load_all(&self) -> Result<Project<String>, anyhow::Error> {
        self.project
            .clone()
            .into_iter()
            .map(|item| {
                item.map_doc(|doc| {
                    let path = self.project_path.join("content").join(&doc.path);
                    let val = fs::read_to_string(path.as_path())
                        .context(format!("Error loading document at {}", path.display()))?;

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
    ) -> anyhow::Result<Project<Option<RenderResult>>> {
        project
            .into_iter()
            .map(|i| i.map_doc(|doc| self.process_document(&doc, format)))
            .collect()
    }

    fn process_document(
        &self,
        doc: &Item<String>,
        format: OutputFormat,
    ) -> anyhow::Result<Option<RenderResult>> {
        let doc = doc.format.loader().load(&doc.content)?;

        if doc.metadata.output.contains(&format) {
            let processor_ctx = ProcessorContext {
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

// fn parse(&self, doc: &Item<()>) -> Result<DocumentParsed, ParserError> {
//         let pattern = self.project_path.as_path().to_str().unwrap().to_string()
//             + "/templates/shortcodes/**/*";
//         let tera = Tera::new(&pattern)?;
//
//         let mut html_preprocessors: Vec<Box<dyn Preprocessor>> = Vec::new();
//         let build_config = self.project_config.build.get_config(&self.mode)?;
//
//         html_preprocessors.push(Box::new(ShortCodeProcessor::new(
//             tera.clone(),
//             "html".to_string(),
//             self.project_config.clone(),
//         )));
//         if build_config.katex_output {
//             html_preprocessors.push(Box::new(KaTeXPreprocessor::new(Opts::default())));
//         }
//
//         let md_preprocessors: Vec<Box<dyn Preprocessor>> = vec![Box::new(ShortCodeProcessor::new(
//             tera,
//             "md".to_string(),
//             self.project_config.clone(),
//         ))];
//
//         let mut parser = DocParser::new(
//             self.project_path.clone(),
//             html_preprocessors,
//             md_preprocessors,
//         )?;
//         parser.parse(doc)
//     }
// }

//
// pub struct Pipeline {
//     mode: String,
//     project_path: PathBuf,
//     project_config: ProjectConfig,
//     renderer: HtmlRenderer,
// }
//
// impl Pipeline {
//     pub fn new<P: AsRef<Path>>(project_path: P, mode: String) -> anyhow::Result<Self> {
//         let config_path = project_path.as_ref().join("config.yml");
//         let config_reader = BufReader::new(File::open(config_path)?);
//
//         let config: ProjectConfig = serde_yaml::from_reader(config_reader)?;
//
//         Ok(Pipeline {
//             mode,
//             project_path: project_path.as_ref().to_path_buf(),
//             renderer: HtmlRenderer::new(project_path.as_ref())?,
//             project_config: config,
//         })
//     }
//
//     fn parse(&self, doc: &Item<()>) -> Result<DocumentParsed, ParserError> {
//         let pattern = self.project_path.as_path().to_str().unwrap().to_string()
//             + "/templates/shortcodes/**/*";
//         let tera = Tera::new(&pattern)?;
//
//         let mut html_preprocessors: Vec<Box<dyn Preprocessor>> = Vec::new();
//         let build_config = self.project_config.build.get_config(&self.mode)?;
//
//         html_preprocessors.push(Box::new(ShortCodeProcessor::new(
//             tera.clone(),
//             "html".to_string(),
//             self.project_config.clone(),
//         )));
//         if build_config.katex_output {
//             html_preprocessors.push(Box::new(KaTeXPreprocessor::new(Opts::default())));
//         }
//
//         let md_preprocessors: Vec<Box<dyn Preprocessor>> = vec![Box::new(ShortCodeProcessor::new(
//             tera,
//             "md".to_string(),
//             self.project_config.clone(),
//         ))];
//
//         let mut parser = DocParser::new(
//             self.project_path.clone(),
//             html_preprocessors,
//             md_preprocessors,
//         )?;
//         parser.parse(doc)
//     }
//
//     pub fn build_file<P: AsRef<Path>>(
//         &mut self,
//         path: P,
//         config: &Project<()>,
//         build_config: &Project<DocumentConfig>,
//     ) {
//         // let doc_base = RelativePathBuf::from_path(&path)?;
//         // let doc_path = doc_base
//         //     .strip_prefix(RelativePathBuf::from_path(&self.project_path)?)
//         //     .unwrap();
//
//         let mut part_id = None;
//         let mut chapter_id = None;
//
//         let doc_path = path
//             .as_ref()
//             .strip_prefix(self.project_path.as_path().join("content"))
//             .unwrap(); // TODO: Error handling;
//         let mut doc_iter = doc_path.iter();
//         let el = doc_iter.next().unwrap().to_str().unwrap();
//
//         let doc = if el.contains('.') {
//             &config.index
//         } else {
//             let first_elem = doc_iter.next().unwrap().to_str().unwrap();
//
//             // let file_name = doc_iter.next().unwrap().to_str().unwrap();
//             let file_id = section_id(path.as_ref()).unwrap();
//
//             let part: &Part<()> = config.content.iter().find(|e| e.id == el).unwrap();
//             let elem = part.chapters.iter().find(|c| c.id == first_elem);
//             part_id = Some(part.id.clone());
//
//             match elem {
//                 None => &part.index,
//                 Some(c) => {
//                     chapter_id = Some(c.id.clone());
//                     let doc = c.documents.iter().find(|d| d.id == file_id);
//                     match doc {
//                         None => &c.index,
//                         Some(d) => d,
//                     }
//                 }
//             }
//         };
//
//         let parsed = self.parse(doc); // TODO: Error message
//         match parsed {
//             Ok(parsed) => {
//                 let parsed_doc = Item {
//                     id: doc.id.clone(),
//                     format: doc.format.clone(),
//                     path: doc.path.clone(),
//                     content: Arc::new(parsed),
//                 };
//
//                 let basebuild_path = self.project_path.join("build");
//                 // let build_path = self.project_path.join("build").join("web");
//                 //
//                 // let mut doc_relative_dir = doc_path.to_path_buf();
//                 // doc_relative_dir.pop();
//                 //
//                 // let build_dir = build_path.join(doc_relative_dir);
//                 // let html = self.renderer.render_document(&parsed, build_config)?;
//                 // // let mut section_build_dir = build_path.join(part.id.clone()).join(chapter.id.clone());
//                 // let section_build_path = build_dir.join(format!("{}.html", doc.id));
//                 //
//                 // fs::create_dir_all(build_dir)?;
//                 // fs::write(section_build_path, html).unwrap();
//
//                 self.write_html(
//                     &parsed_doc,
//                     &part_id,
//                     &chapter_id,
//                     build_config,
//                     &basebuild_path,
//                 )
//                 .unwrap(); // TODO: Error handling
//                 self.write_notebook(&parsed_doc, &basebuild_path).unwrap(); // TODO: Error handling
//
//                 println!("🔔 Document {} changed, re-rendered output", doc.id);
//             }
//             Err(e) => {
//                 println!("Error {}", e);
//             }
//         }
//     }
//
//     pub fn build_everything(
//         &mut self,
//         config: Project<()>,
//     ) -> anyhow::Result<Project<DocumentConfig>> {
//         println!("[2/4] 📖 Parsing source documents...");
//         let parsed: Project<DocumentParsed> = config
//             .clone()
//             .into_iter()
//             .filter_map(|item| {
//                 let res = item.map_doc(|doc| self.parse(&doc));
//                 let res = match res {
//                     Ok(i) => Some(i),
//                     Err(e) => {
//                         let ei: &dyn Error = &e;
//                         println!(
//                             "{}{}error: {}{}{}",
//                             style::Bold,
//                             color::Fg(color::Red),
//                             style::Reset,
//                             color::Fg(color::Reset),
//                             ei
//                         );
//                         println!("\t{}", e);
//                         if let Some(source) = ei.source() {
//                             println!("\t\tCaused by: {}", source);
//                         }
//
//                         None
//                     }
//                 };
//                 res
//             })
//             .collect::<Project<DocumentParsed>>();
//
//         // Work on how to create build configuration
//         println!("[3/4] 🌵 Generating build configuration...");
//         let build_config: Project<DocumentConfig> = parsed
//             .clone()
//             .into_iter()
//             .map(|item| {
//                 item.map_doc(|doc| {
//                     let c = doc.content;
//                     Ok(DocumentConfig {
//                         id: doc.id,
//                         header: c.title.clone(),
//                         frontmatter: c.frontmatter.clone(),
//                     })
//                 })
//             })
//             .collect::<anyhow::Result<Project<DocumentConfig>>>()?;
//
//         let build_path = self.project_path.join("build");
//
//         if build_path.as_path().exists() {
//             fs::remove_dir_all(build_path.as_path())?;
//         }
//
//         fs::create_dir(build_path.as_path())?;
//
//         println!("[X/4] Writing notebooks...");
//         let _notebook_errors: Vec<()> = parsed
//             .clone()
//             .into_iter()
//             .map(|item| {
//                 if item.doc.content.frontmatter.output.source {
//                     self.write_notebook(&item.doc, &build_path)
//                 } else {
//                     Ok(())
//                 }
//             })
//             .collect::<anyhow::Result<Vec<()>>>()?;
//
//         println!("[4/4] 🌼 Rendering output...");
//         let html_results_filtered = parsed
//             .into_iter()
//             .map(|item| {
//                 if item.doc.content.frontmatter.output.web {
//                     // Only output if active (TODO: Don't parse html if not necessary)
//                     self.write_html(
//                         &item.doc,
//                         &item.part_id,
//                         &item.chapter_id,
//                         &build_config,
//                         &build_path,
//                     )
//                     .map(|_| item)
//                 } else {
//                     Ok(item)
//                 }
//             })
//             .filter_map(|result| match result {
//                 Ok(i) => Some(i),
//                 Err(e) => {
//                     match e {
//                         HtmlRenderError::TemplateError(e, title) => {
//                             println!(
//                                 "{}[Error] {}Could not render '{}' due to:",
//                                 color::Fg(color::Red),
//                                 color::Fg(color::Black),
//                                 title
//                             );
//                             println!("\t{}", e);
//                             if let Some(source) = e.source() {
//                                 println!("\t\tCaused by: {}", source);
//                             }
//                         }
//                         HtmlRenderError::Other(e, title) => {
//                             println!("[Error] Could not render '{}' due to:", title);
//                             println!("[Error] {}", e);
//                         }
//                     }
//                     None
//                 }
//             });
//
//         let _md_errors: Vec<ProjectItem<DocumentParsed>> = html_results_filtered
//             // .clone()
//             .map(|item| self.write_markdown(&item.doc, &build_path).map(|_| item))
//             .filter_map(|result| match result {
//                 Ok(i) => Some(i),
//                 Err(e) => {
//                     println!("[Error] Markdown render error: {}", e);
//                     None
//                 }
//             })
//             .collect();
//
//         println!("[5/4]  Copying resources...");
//         let resource_path = self.project_path.as_path().join("resources");
//         let path_web = build_path.as_path().join("web");
//         let resource_path_build_web = path_web.as_path().join("resources");
//
//         for part in config.content {
//             for chapter in part.chapters {
//                 chapter
//                     .files
//                     .into_iter()
//                     .map(|path| {
//                         let relative =
//                             path.strip_prefix(self.project_path.as_path().join("content"))?;
//                         // let web_path = build_path.as_path().join("web");
//                         let source_path = build_path.as_path().join("source");
//
//                         let to_path = source_path.join(relative);
//                         let mut to_dir = to_path.clone();
//                         to_dir.pop();
//
//                         fs::create_dir_all(to_dir)?;
//
//                         let content = fs::read_to_string(path.as_path())?;
//                         let task = parse_code_string(&content)?;
//                         let out = task.write_string(false);
//
//                         fs::write(to_path, out)?;
//
//                         // fs::copy(path.as_path(), to_path)?;
//                         Ok(())
//                     })
//                     .collect::<anyhow::Result<Vec<()>>>()?;
//             }
//         }
//
//         let mut options = fs_extra::dir::CopyOptions::new();
//         options.overwrite = true;
//
//         fs::create_dir_all(resource_path_build_web.as_path())?;
//         fs_extra::copy_items(&[resource_path], path_web, &options)?;
//
//         Ok(build_config)
//     }
//
//     fn write_notebook<P: AsRef<Path>>(
//         &self,
//         doc: &Item<DocumentParsed>,
//         build_path: P,
//     ) -> anyhow::Result<()> {
//         let mut notebook_build_dir = build_path.as_ref().join("source").join(&doc.path);
//         notebook_build_dir.pop(); // Pop filename
//         let notebook_build_path = notebook_build_dir.join(format!("{}.ipynb", doc.id));
//
//         fs::create_dir_all(notebook_build_dir)?;
//         let f = File::create(notebook_build_path)?;
//         let writer = BufWriter::new(f);
//         serde_json::to_writer(writer, &doc.content.notebook)?;
//         Ok(())
//     }
//
//     fn write_markdown<P: AsRef<Path>>(
//         &self,
//         doc: &Item<DocumentParsed>,
//         build_path: P,
//     ) -> anyhow::Result<()> {
//         let mut md_build_dir = build_path.as_ref().join("md").join(&doc.path);
//         md_build_dir.pop(); // Pop filename
//         let md_build_path = md_build_dir.join(format!("{}.md", doc.id));
//
//         fs::create_dir_all(md_build_dir)?;
//         fs::write(md_build_path, &doc.content.md).unwrap();
//         Ok(())
//     }
//
//     fn write_html<P: AsRef<Path>>(
//         &self,
//         doc: &Item<DocumentParsed>,
//         part_id: &Option<String>,
//         chapter_id: &Option<String>,
//         build_config: &Project<DocumentConfig>,
//         build_path: P,
//     ) -> Result<(), HtmlRenderError> {
//         let output = self
//             .renderer
//             .render_document(&doc.content, &doc.id, part_id, chapter_id, build_config)
//             .map_err(|e| TemplateError(e, doc.path.to_str().unwrap().to_string()))?;
//
//         let mut html_build_dir = build_path.as_ref().join("web").join(&doc.path);
//         html_build_dir.pop(); // Pop filename
//         let section_build_path = html_build_dir.join(format!("{}.html", doc.id));
//
//         fs::create_dir_all(html_build_dir)
//             .context("Could not create directory")
//             .map_err(|e| HtmlRenderError::Other(e, doc.path.to_str().unwrap().to_string()))?;
//         fs::write(section_build_path, output).unwrap();
//
//         Ok(())
//     }
// }
