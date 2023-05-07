use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};

use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Context as AContext};

use clap::ValueEnum;
use console::style;
use image::ImageOutputFormat;
use indicatif::{
    MultiProgress, ParallelProgressIterator, ProgressBar, ProgressIterator, ProgressStyle,
};
use serde_json::{from_value, to_value, Value};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tera::{Context, Filter, Function};

use cdoc::ast::Ast;
use cdoc::config::Format;
use cdoc::document::{split_shortcodes, Document, DocumentMetadata};
use cdoc::processors::PreprocessorContext;

use cdoc::renderers::{DocumentRenderer, RenderContext, RenderResult};
use cdoc::templates::TemplateManager;
use image::io::Reader as ImageReader;
use mover::{MoveContext, Mover};
use serde::{Deserialize, Serialize};

use cdoc::renderers;
use cdoc::renderers::generic::GenericRenderer;
use rayon::prelude::*;

use crate::generators::Generator;
use crate::project::config::ProjectConfig;
use crate::project::{
    section_id, ItemDescriptor, Part, Project, ProjectItem, ProjectItemVec, ProjectResult,
};
use std::borrow::Borrow;

mod mover;

fn create_embed_fn(resource_path: PathBuf, cache_path: PathBuf) -> impl Filter {
    Box::new(
        move |url: &Value, _args: &HashMap<String, Value>| -> tera::Result<Value> {
            match from_value::<String>(url.clone()) {
                Ok(v) => {
                    let mut file_no_ext = PathBuf::from_str(&v).unwrap();
                    file_no_ext.set_extension(".txt");

                    let cache_file = cache_path.join(&file_no_ext);
                    let resource_file = resource_path.join(v);
                    let resource_meta = resource_file.metadata()?;

                    let data = match cache_file.metadata().ok().and_then(|meta| {
                        (meta.modified().unwrap() > resource_meta.modified().unwrap()).then_some(())
                    }) {
                        None => {
                            let img = ImageReader::open(&resource_file)
                                .map_err(|_| tera::Error::msg("Could not open image"))?
                                .decode()
                                .map_err(|_| tera::Error::msg("Could not decode image"))?;
                            // println!("loaded");
                            let mut image_data: Vec<u8> = Vec::new();
                            let mut img_writer = BufWriter::new(Cursor::new(&mut image_data));
                            img.write_to(&mut img_writer, ImageOutputFormat::Jpeg(60))
                                .map_err(|_| tera::Error::msg("Could not write image data"))?;
                            drop(img_writer);
                            // println!("semi");
                            let data = base64_simd::STANDARD.encode_to_string(&image_data);

                            fs::create_dir_all(cache_file.parent().unwrap())?;
                            fs::write(cache_file, &data)?;
                            data
                        }
                        Some(_) => fs::read_to_string(&cache_file).unwrap(),
                    };

                    // println!("written");
                    Ok(to_value(data).unwrap())
                }
                Err(_) => Err("file not found".into()),
            }
        },
    )
}

#[derive(Clone)]
pub struct Pipeline {
    #[allow(unused)]
    pub mode: Mode,
    pub project_path: PathBuf,
    pub project: Project<()>,
    pub project_config: ProjectConfig,
    templates: TemplateManager,

    cached_contexts: Arc<Mutex<HashMap<String, ProjectItemVec>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, ValueEnum, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Release,
    Draft,
}

pub fn print_err<T>(res: anyhow::Result<T>) -> Option<T> {
    match res {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("{} {}", style("Error:").red().bold(), e);
            e.chain()
                .skip(1)
                .for_each(|cause| eprintln!(" {} {}", style("caused by:").bold(), cause));
            None
        }
    }
}

impl Pipeline {
    pub fn new<P: AsRef<Path>>(
        project_path: P,
        mode: Mode,
        config: ProjectConfig,
        project: Project<()>,
    ) -> anyhow::Result<Self> {
        print!("Parsing templates... ");
        let mut template_manager = TemplateManager::from_path(
            project_path.as_ref().join("templates"),
            project_path.as_ref().join("filters"),
        )?;
        println!("{}", style("done").green());

        let cache_path = project_path.as_ref().join(".cache");
        fs::create_dir_all(&cache_path)?;

        template_manager.register_filter(
            "embed",
            create_embed_fn(project_path.as_ref().join("resources"), cache_path),
        );

        let mut pipeline = Pipeline {
            mode,
            project_path: project_path.as_ref().to_path_buf(),
            project,
            project_config: config,
            templates: template_manager,
            cached_contexts: Arc::new(Mutex::new(HashMap::new())),
        };

        let p2 = pipeline.clone();

        pipeline
            .templates
            .tera
            .register_function("render", p2.create_render_source());

        Ok(pipeline)
    }

    fn create_render_source(self) -> impl Function {
        Box::new(
            move |args: &HashMap<String, Value>| -> tera::Result<Value> {
                let mut counters = HashMap::new();
                let val = args
                    .get("body")
                    .ok_or(tera::Error::msg("missing argument 'body'"))?;
                if let Value::String(s) = val {
                    let ast =
                        split_shortcodes(s, &mut counters).map_err(|e| tera::Error::msg(e))?;
                    let doc = Document::new(Ast(ast), DocumentMetadata::default(), HashMap::new());

                    let fstring = args
                        .get("format")
                        .ok_or(tera::Error::msg("missing argument 'format'"))?
                        .to_string();
                    let format: Box<dyn Format> = serde_json::from_str(&format!(
                        "{{\"{}\": {{}}}}",
                        &fstring[1..fstring.len() - 1]
                    ))
                    .expect("problems!");

                    let mut ctx = self.get_render_context(&doc, format.borrow());
                    let mut renderer = GenericRenderer;
                    let res = renderer
                        .render_doc(&mut ctx)
                        .map_err(|e| tera::Error::msg(e))?;
                    let val = res.content;
                    Ok(Value::String(val))
                } else {
                    Err(tera::Error::msg("invalid type for 'body'"))
                }
            },
        )
    }

    fn get_render_context<'a>(
        &'a self,
        doc: &'a Document<Ast>,
        format: &'a dyn Format,
    ) -> RenderContext<'a> {
        let mut meta = Context::default();
        meta.insert("config", &self.project_config);
        let ts = ThemeSet::load_defaults();
        RenderContext {
            templates: &self.templates,
            extra_args: meta,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme: ts.themes["base16-ocean.light"].clone(),
            notebook_output_meta: &self.project_config.notebook_meta.as_ref().unwrap(),
            format,
            doc,
            ids: &doc.ids,
            ids_map: &doc.id_map,
        }
    }

    fn get_build_path(&self, format: &dyn Format) -> PathBuf {
        self.project_path.join("build").join(format.name())
    }

    pub fn reload_templates(&mut self) -> anyhow::Result<()> {
        self.templates.reload()
    }

    pub fn build_single(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let relpath = path.strip_prefix(self.project_path.join("content"))?;
        println!("{} {}", style("Building file").bold(), relpath.display());
        println!("{}", style("-".repeat(60)).blue());
        let item = self.doc_from_path(path)?;
        let item2 = item.clone();

        let loaded = item.map_doc(|doc| {
            let path = self.project_path.join("content").join(doc.path);
            let val = fs::read_to_string(path.as_path())
                .context(format!("Error loading document at {}", path.display()))?;
            Ok::<String, anyhow::Error>(val)
        })?;

        let mut all_errors = Vec::new();

        for format in self.project_config.outputs.clone() {
            print!("format: {}", style(&format).bold());
            let output = self.process_document(&loaded.doc, format.as_ref());

            match output {
                Err(e) => {
                    all_errors.push(e);
                    println!(" {}", style("error").red());
                }
                Ok(output) => {
                    if let Some(output) = output {
                        let project = self
                            .cached_contexts
                            .lock()
                            .unwrap()
                            .get(format.name())
                            .ok_or_else(|| anyhow!("Cached context is missing"))?
                            .clone();

                        let project_vec =
                            self.update_cache(&item2, format.as_ref(), &output, project.clone());

                        let ctx = Generator {
                            root: self.project_path.clone(),
                            project_vec: &project_vec,
                            project: project_vec.iter().collect(),
                            templates: &self.templates,
                            config: self.project_config.clone(),
                            mode: self.mode,
                            build_dir: self.get_build_path(format.as_ref()),
                            format: format.as_ref(),
                        };
                        ctx.generate_single(&output, &item2)?;

                        println!(" {}", style("done").green());
                    } else {
                        println!(" {}", style("no output").yellow());
                    }
                }
            }
            // let output = print_err(output).flatten();
        }

        println!("{}", style("-".repeat(60)).blue());
        if all_errors.is_empty() {
            println!("{}", style("Success").green().bold());
        } else {
            let len = all_errors.len();
            all_errors.into_iter().for_each(|e| {
                eprintln!("{} {}", style("Error:").red().bold(), e);
                e.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!(" {} {}", style("caused by:").bold(), cause));
            });
            println!("{}", style("-".repeat(60)).blue());

            println!(
                "{}",
                style(format!(
                    "File built with non-critical errors ({} total)",
                    len
                ))
                .yellow()
                .bold()
            );
        }

        Ok(())
    }

    fn update_cache(
        &mut self,
        item2: &ItemDescriptor<()>,
        format: &dyn Format,
        output: &Document<RenderResult>,
        mut project: ProjectItemVec,
    ) -> ProjectItemVec {
        let i3 = item2.clone();

        let item = project
            .iter_mut()
            .find(|item| {
                let part = item.part_idx.and_then(|i| i3.part_idx.map(|j| i == j));
                let chapter = item
                    .chapter_idx
                    .and_then(|i| i3.chapter_idx.map(|j| i == j));
                let doc = item.doc_idx.and_then(|i| i3.doc_idx.map(|j| i == j));
                // let combined = chapter.and_then(part);
                match part {
                    Some(is_part) => {
                        is_part
                            && match chapter {
                                None => true,
                                Some(is_chapter) => {
                                    is_chapter
                                        && match doc {
                                            None => true,
                                            Some(is_doc) => is_doc,
                                        }
                                }
                            }
                    }
                    None => true,
                }
            })
            .unwrap();
        item.doc.content = Arc::new(Some(output.clone()));

        self.cached_contexts
            .lock()
            .unwrap()
            .insert(format.name().to_string(), project.clone());
        project
    }

    fn doc_from_path(&self, path: PathBuf) -> anyhow::Result<ItemDescriptor<()>> {
        let mut part_id = None;
        let mut chapter_id = None;
        let mut part_idx = None;
        let mut chapter_idx = None;
        let mut doc_idx = None;

        let doc_path = path
            .as_path()
            .strip_prefix(self.project_path.as_path().join("content"))?; // TODO: Error handling;
        let mut doc_iter = doc_path.iter();
        let el = doc_iter.next().unwrap().to_str().unwrap();

        let doc = if el.contains('.') {
            &self.project.index
        } else {
            let first_elem = doc_iter
                .next()
                .ok_or(anyhow!(
                    "Empty part. Parts must contain index.ms or index.ipynb"
                ))?
                .to_str()
                .unwrap();

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

            let pid = self
                .project
                .content
                .iter()
                .position(|e| e.id == el)
                .expect("Part index not found");
            part_idx = Some(pid);

            match elem {
                None => &part.index,
                Some(c) => {
                    chapter_id = Some(c.id.clone());
                    let cid = part
                        .chapters
                        .iter()
                        .position(|c| c.id == first_elem)
                        .expect("Part index not found");
                    chapter_idx = Some(cid);
                    let doc = c.documents.iter().find(|d| d.id == file_id);
                    match doc {
                        None => &c.index,
                        Some(d) => {
                            let did = c
                                .documents
                                .iter()
                                .position(|d| d.id == file_id)
                                .expect("Part index not found");
                            doc_idx = Some(did);

                            d
                        }
                    }
                }
            }
        };

        let item = ItemDescriptor {
            part_id,
            chapter_id,
            part_idx,
            chapter_idx,
            doc: doc.clone(),
            doc_idx,
            files: None,
        };

        Ok(item)
    }

    pub fn build_all(&mut self, remove_existing: bool) -> Result<(), anyhow::Error> {
        let build_path = self.project_path.join("build");

        if remove_existing && build_path.exists() {
            fs::remove_dir_all(build_path)?;
        }

        let loaded = self.load_files()?;

        println!("{}", style("=".repeat(60)).blue());
        println!(
            "{} ({} files)",
            style("Building project").bold(),
            loaded.len()
        );
        println!("{}", style("-".repeat(60)).blue());

        let mut all_errs = Arc::new(Mutex::new(Vec::new()));

        let mut multi = MultiProgress::new();
        let mut bars = Vec::new();

        let bar_len = self.project.len() * 2;
        let sty = ProgressStyle::with_template("{msg:<20} {pos}/{len} {bar:20.cyan/blue}")?;

        for f in &self.project_config.outputs {
            let p = ProgressBar::new(bar_len as u64);
            let bar = multi.add(p);
            bar.set_style(sty.clone());

            bars.push(bar);
        }

        self.project_config
            .outputs
            .par_iter()
            .zip(bars.clone())
            .for_each(|(format, bar)| {
                let mut format_errs = Vec::new();

                bar.set_message(format!(
                    "{} {}",
                    style(format.name()).bold(),
                    style("parsing").blue()
                ));
                let (output, mut errs) =
                    self.process_all(loaded.clone(), format.as_ref(), bar.clone());

                format_errs.append(&mut errs.lock().unwrap());
                let context = Generator {
                    root: self.project_path.to_path_buf(),
                    project_vec: &output,
                    project: output.iter().collect(),
                    mode: self.mode,
                    templates: &self.templates,
                    config: self.project_config.clone(),
                    format: format.as_ref(),
                    build_dir: self.get_build_path(format.as_ref()),
                };
                self.cached_contexts
                    .lock()
                    .unwrap()
                    .insert(format.name().to_string(), output.clone());

                bar.set_message(format!(
                    "{} {}",
                    style(format.name()).bold(),
                    style("writing").blue()
                ));
                let res = context
                    .generate(bar.clone())
                    .with_context(|| format!("Could not generate {}", format));

                if let Err(e) = res {
                    format_errs.push(e);
                }

                // Move extra files

                // print!(", copying additional files");
                let move_ctx = MoveContext {
                    project_path: self.project_path.to_path_buf(),
                    build_dir: self.get_build_path(format.as_ref()),
                    settings: self.project_config.parser.settings.clone(),
                };

                let res =
                    Mover::traverse_dir(self.project_path.join("content").to_path_buf(), &move_ctx);
                if let Err(e) = res {
                    format_errs.push(e);
                }

                // Error display
                if format_errs.is_empty() {
                    bar.finish_with_message(format!(
                        "{} {}",
                        style(format.name()).bold(),
                        style("success").green()
                    ));
                    // println!("{}", style("success").green());
                } else {
                    bar.finish_with_message(format!(
                        "{} {}",
                        style(format.name()).bold(),
                        style(format!("({} errors)", format_errs.len())).red()
                    ));
                }

                all_errs.lock().unwrap().append(&mut format_errs);
            });

        let all_errs = all_errs.lock().unwrap();

        println!("{}", style("-".repeat(60)).blue());
        if all_errs.is_empty() {
            println!("{}", style("Project built without errors").green().bold());
        } else {
            let len = all_errs.len();
            all_errs.iter().for_each(|e| {
                eprintln!("{} {}", style("Error:").red().bold(), e);
                e.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!(" {} {}", style("caused by:").bold(), cause));
            });
            println!("{}", style("-".repeat(60)).blue());

            println!(
                "{}",
                style(format!(
                    "Project built with non-critical errors ({} total)",
                    len
                ))
                .yellow()
                .bold()
            );
        }
        println!("{}", style("=".repeat(60)).blue());

        Ok(())
    }

    fn load_files(&self) -> Result<Project<String>, anyhow::Error> {
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

    fn process_all(
        &self,
        project: Project<String>,
        format: &dyn Format,
        bar: ProgressBar,
    ) -> (
        Vec<ItemDescriptor<Option<Document<RenderResult>>>>,
        Arc<Mutex<Vec<anyhow::Error>>>,
    ) {
        let mut errs = Arc::new(Mutex::new(Vec::new()));

        let project_vec: Vec<ItemDescriptor<String>> = project.into_iter().collect();

        let res = project_vec
            .into_par_iter()
            .progress_with(bar)
            .map(|i| {
                let res = self.process_document(&i.doc, format).with_context(|| {
                    format!(
                        "Failed to process document – {}",
                        style(format!("content/{}", i.doc.path.display())).italic()
                    )
                });

                let res = match res {
                    Ok(good) => good,
                    Err(e) => {
                        let mut errs_guard = errs.lock().unwrap();
                        errs_guard.push(e);
                        None
                    }
                };

                // let res = print_err(res);

                ItemDescriptor {
                    part_id: i.part_id,
                    chapter_id: i.chapter_id,
                    part_idx: i.part_idx,
                    chapter_idx: i.chapter_idx,
                    doc_idx: i.doc_idx,
                    doc: ProjectItem {
                        id: i.doc.id,
                        format: i.doc.format,
                        path: i.doc.path,
                        content: Arc::new(res),
                    },
                    files: i.files,
                }
            })
            .collect::<Vec<ItemDescriptor<Option<Document<RenderResult>>>>>();

        // pb.finish_with_message(format!("Done"));

        (res, errs)
    }

    fn process_document(
        &self,
        item: &ProjectItem<String>,
        format: &dyn Format,
    ) -> anyhow::Result<Option<Document<RenderResult>>> {
        let doc = item.format.loader().load(&item.content)?;

        if format.no_parse() {
            Ok(Some(Document {
                content: "".to_string(),
                metadata: doc.metadata,
                variables: doc.variables,
                ids: doc.ids,
                id_map: doc.id_map,
            }))
        } else if !doc
            .metadata
            .exclude_outputs
            .as_ref()
            .map(|o| o.contains(&format.name().to_string()))
            .unwrap_or_default()
        {
            let processor_ctx = PreprocessorContext {
                templates: &self.templates,
                output_format: format,
            };

            let res = self.project_config.parser.parse(&doc, &processor_ctx)?;

            // let res = print_err(res)?;

            let ctx = self.get_render_context(&res, format);
            let mut renderer = format.renderer();

            Ok(Some(renderer.render_doc(&ctx)?))
        } else {
            Ok(None)
        }
    }
}
