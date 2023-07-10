use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};

use std::str::FromStr;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context as AContext};

use console::style;
use image::ImageOutputFormat;
use indicatif::{MultiProgress, ParallelProgressIterator, ProgressBar, ProgressStyle};
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

use cdoc::renderers::generic::GenericRenderer;
use rayon::prelude::*;

use crate::generators::Generator;
use crate::project::config::{Profile, ProjectConfig};
use crate::project::{
    from_vec, section_id, ContentItem, ContentItemDescriptor, ContentResult, DocumentDescriptor,
    ItemDescriptor, Part, Project, ProjectItem, ProjectItemVec,
};
use lazy_static::lazy_static;
use std::borrow::Borrow;

mod mover;

fn create_embed_fn(resource_path: PathBuf, cache_path: PathBuf) -> impl Filter {
    Box::new(
        move |url: &Value, _args: &HashMap<String, Value>| -> tera::Result<Value> {
            match from_value::<String>(url.clone()) {
                Ok(v) => {
                    let mut file_no_ext = PathBuf::from_str(&v).unwrap();
                    if file_no_ext.extension().unwrap().to_str().unwrap() == "svg" {
                        let contents = fs::read_to_string(resource_path.join(v)).unwrap();
                        Ok(to_value(contents).unwrap())
                    } else {
                        file_no_ext.set_extension(".txt");

                        let cache_file = cache_path.join(&file_no_ext);
                        let resource_file = resource_path.join(v);
                        let resource_meta = resource_file.metadata()?;

                        let data = match cache_file.metadata().ok().and_then(|meta| {
                            (meta.modified().unwrap() > resource_meta.modified().unwrap())
                                .then_some(())
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
                }
                Err(_) => Err("file not found".into()),
            }
        },
    )
}

/// Orchestrates the build process for a project (either everything or single files).
#[derive(Clone)]
pub struct Pipeline {
    /// Build profile used for output generation
    pub profile: Profile,
    pub profile_name: String,
    /// Project root path
    pub project_path: PathBuf,
    /// Project structure
    pub project: Project<()>,
    pub project_structure: ContentItem<()>,
    /// Configuration for project loaded from config.yml
    pub project_config: ProjectConfig,

    templates: TemplateManager,
    cached_contexts: Arc<Mutex<HashMap<String, ProjectItemVec>>>,
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

lazy_static! {
    static ref DEFAULT_SYNTAX: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref DEFAULT_THEME: ThemeSet = ThemeSet::load_defaults();
}

impl Pipeline {
    /// Create a new pipeline. Initializes templates.
    pub fn new<P: AsRef<Path>>(
        project_path: P,
        profile: String,
        config: ProjectConfig,
        project: Project<()>,
        project_structure: ContentItem<()>,
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

        let p = config
            .profiles
            .get(&profile)
            .ok_or(anyhow!("Profile doesn't exist"))?
            .clone();

        let mut pipeline = Pipeline {
            profile: p,
            profile_name: profile,
            project_path: project_path.as_ref().to_path_buf(),
            project,
            project_structure,
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
                    let ast = split_shortcodes(s, &mut counters).map_err(tera::Error::msg)?;
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

                    let ctx = self.get_render_context(&doc, format.borrow());
                    let mut renderer = GenericRenderer::default();
                    let res = renderer.render_doc(&ctx).map_err(tera::Error::msg)?;
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
        meta.insert("ids", &doc.ids);
        meta.insert("id_map", &doc.id_map);
        meta.insert("doc_meta", &doc.metadata);
        let ts = &DEFAULT_THEME;
        RenderContext {
            templates: &self.templates,
            extra_args: meta,
            syntax_set: &DEFAULT_SYNTAX,
            theme: &ts.themes["base16-ocean.light"],
            notebook_output_meta: self.project_config.notebook_meta.as_ref().unwrap(),
            format,
            doc,
        }
    }

    fn get_build_path(&self, format: &dyn Format) -> PathBuf {
        self.project_path
            .join("build")
            .join(&self.profile_name)
            .join(format.name())
    }

    pub fn reload_templates(&mut self) -> anyhow::Result<()> {
        self.templates.reload()
    }

    /// Build a single content file.
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
                            project: from_vec(&project_vec),
                            templates: &self.templates,
                            config: self.project_config.clone(),
                            mode: self.profile.mode,
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
        item2: &ContentItemDescriptor<()>,
        format: &dyn Format,
        output: &Document<RenderResult>,
        mut project: ProjectItemVec,
    ) -> ProjectItemVec {
        let i3 = item2.clone();

        let item = project
            .iter_mut()
            .find(|item| item.path == item2.path)
            .unwrap();
        item.doc.content = Arc::new(Some(output.clone()));

        self.cached_contexts
            .lock()
            .unwrap()
            .insert(format.name().to_string(), project.clone());
        project
    }

    fn doc_from_path(&self, path: PathBuf) -> anyhow::Result<ContentItemDescriptor<()>> {
        let doc_path = path
            .as_path()
            .strip_prefix(self.project_path.as_path().join("content"))?; // TODO: Error handling;

        let path: Vec<String> = doc_path
            .into_iter()
            .map(|d| d.to_str().unwrap().split(".").next().unwrap().to_string())
            .collect();

        let path_idx = self
            .project_structure
            .get_path_idx(&path[..])
            .ok_or(anyhow!("Path is invalid"))?;

        Ok(ContentItemDescriptor {
            path,
            path_idx: path_idx.clone(),
            doc: self.project_structure.doc_at_idx(&path_idx[..])?.clone(),
        })

        // let mut part_id = None;
        // let mut chapter_id = None;
        // let mut part_idx = None;
        // let mut chapter_idx = None;
        // let mut doc_idx = None;
        //
        // let mut doc_iter = doc_path.iter();
        // let el = doc_iter.next().unwrap().to_str().unwrap();
        //
        // let doc = if el.contains('.') {
        //     &self.project.index
        // } else {
        //     let first_elem = doc_iter
        //         .next()
        //         .ok_or(anyhow!(
        //             "Empty part. Parts must contain index.ms or index.ipynb"
        //         ))?
        //         .to_str()
        //         .unwrap();
        //
        //     // let file_name = doc_iter.next().unwrap().to_str().unwrap();
        //     let file_id = section_id(path.as_path()).unwrap();
        //
        //     let part: &Part<()> = self
        //         .project
        //         .content
        //         .iter()
        //         .find(|e| e.id == el)
        //         .expect("Part not found for single document");
        //     let elem = part.chapters.iter().find(|c| c.id == first_elem);
        //     part_id = Some(part.id.clone());
        //
        //     let pid = self
        //         .project
        //         .content
        //         .iter()
        //         .position(|e| e.id == el)
        //         .expect("Part index not found");
        //     part_idx = Some(pid + 1);
        //
        //     match elem {
        //         None => &part.index,
        //         Some(c) => {
        //             chapter_id = Some(c.id.clone());
        //             let cid = part
        //                 .chapters
        //                 .iter()
        //                 .position(|c| c.id == first_elem)
        //                 .expect("Part index not found");
        //             chapter_idx = Some(cid + 1);
        //             let doc = c.documents.iter().find(|d| d.id == file_id);
        //             match doc {
        //                 None => &c.index,
        //                 Some(d) => {
        //                     let did = c
        //                         .documents
        //                         .iter()
        //                         .position(|d| d.id == file_id)
        //                         .expect("Part index not found");
        //                     doc_idx = Some(did + 1);
        //
        //                     d
        //                 }
        //             }
        //         }
        //     }
        // };
        //
        // let item = ItemDescriptor {
        //     part_id,
        //     chapter_id,
        //     part_idx,
        //     chapter_idx,
        //     doc: doc.clone(),
        //     doc_idx,
        //     files: None,
        // };
        //
        // Ok(item)
    }

    /// Build the whole project (optionally removes existing build)
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

        let all_errs = Arc::new(Mutex::new(Vec::new()));

        let multi = MultiProgress::new();
        let mut bars = Vec::new();

        let bar_len = self.project.len() * 2;
        let sty = ProgressStyle::with_template("{msg:<20} {pos}/{len} {bar:20.cyan/blue}")?;

        for _f in &self.project_config.outputs {
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
                let (output, errs) = self.process_all(loaded.clone(), format.as_ref(), bar.clone());
                // println!(
                //     "{:#?}",
                //     output
                //         .clone()
                //         .into_iter()
                //         .map(|i| (i.path, i.path_idx))
                //         .collect::<Vec<(Vec<String>, Vec<usize>)>>()
                // );
                // println!("{:#?}", output.iter().collect::<ContentResult>());
                format_errs.append(&mut errs.lock().unwrap());
                let context = Generator {
                    root: self.project_path.to_path_buf(),
                    project_vec: &output,
                    project: from_vec(&output),
                    mode: self.profile.mode,
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
                    settings: self.profile.parser.settings.clone(),
                };

                let res = Mover::traverse_dir(self.project_path.join("content"), &move_ctx);
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

    fn load_files(&self) -> anyhow::Result<Vec<ContentItemDescriptor<String>>> {
        self.project_structure
            .clone()
            .to_vector()
            .into_iter()
            .map(|item| {
                item.map_doc(|doc| {
                    let path = self.project_path.join("content").join(doc.path);
                    let val = fs::read_to_string(path.as_path())
                        .context(format!("Error loading document {}", path.display()))?;

                    Ok(val)
                })
            })
            .collect::<anyhow::Result<Vec<ContentItemDescriptor<String>>>>()
    }

    fn process_all(
        &self,
        project: Vec<ContentItemDescriptor<String>>,
        format: &dyn Format,
        bar: ProgressBar,
    ) -> (ProjectItemVec, Arc<Mutex<Vec<anyhow::Error>>>) {
        let errs = Arc::new(Mutex::new(Vec::new()));

        // let project_vec: Vec<ItemDescriptor<String>> = project.into_iter().collect();
        // let project_structure_vec

        let res = project
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
                ContentItemDescriptor {
                    path: i.path,
                    path_idx: i.path_idx,
                    doc: DocumentDescriptor {
                        id: i.doc.id,
                        format: i.doc.format,
                        path: i.doc.path,
                        content: Arc::new(res),
                    },
                }
            })
            .collect::<Vec<ContentItemDescriptor<Option<Document<RenderResult>>>>>();

        // pb.finish_with_message(format!("Done"));

        (res, errs)
    }

    fn process_document(
        &self,
        item: &DocumentDescriptor<String>,
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

            let res = self.profile.parser.parse(&doc, &processor_ctx)?;

            // let res = print_err(res)?;

            let ctx = self.get_render_context(&res, format);
            let mut renderer = format.renderer();

            Ok(Some(renderer.render_doc(&ctx)?))
        } else {
            Ok(None)
        }
    }
}
