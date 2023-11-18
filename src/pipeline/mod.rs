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

use cdoc::config::Format;

use cdoc::preprocessors::PreprocessorContext;

use cdoc::renderers::{DocumentRenderer, RenderContext, RenderResult};
use cdoc::templates::TemplateManager;
use image::io::Reader as ImageReader;
use mover::Mover;

use rayon::prelude::*;

use crate::generators::Generator;
use crate::project::config::{Mode, Profile, ProjectConfig};
use crate::project::{
    from_vec, ContentItem, ContentItemDescriptor, DocumentDescriptor, ProjectItemContentVec,
    ProjectItemVec, ProjectItemVecErr,
};

use crate::project::caching::Cache;
use cdoc::renderers::base::{ElementRenderer, ElementRendererConfig};
use cdoc::renderers::extensions::build_extensions;

use cdoc_base::document::{Document, Metadata};
use cowstr::CowStr;
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
    pub project_structure: ContentItem<()>,
    /// Configuration for project loaded from config.yml
    pub project_config: ProjectConfig,

    pub cache_info: Cache,

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
        project_structure: ContentItem<()>,
    ) -> anyhow::Result<Self> {
        let p = config
            .profiles
            .get(&profile)
            .ok_or(anyhow!("Profile doesn't exist"))?
            .clone();

        print!("Parsing templates... ");
        let mut template_manager = TemplateManager::from_path(
            project_path.as_ref().join("templates"),
            project_path.as_ref().join("filters"),
            p.create_filters,
        )?;
        println!("{}", style("done").green());

        let cache_path = project_path.as_ref().join(".cache");
        fs::create_dir_all(&cache_path)
            .with_context(|| format!("at path {}", cache_path.display()))?;

        template_manager.register_filter(
            "embed",
            create_embed_fn(project_path.as_ref().join("resources"), cache_path.clone()),
        );

        let cache_info = match fs::read_to_string(cache_path.join("project_info.json")) {
            Ok(cache_val) => serde_json::from_str(&cache_val)?,
            Err(_) => Cache::default(),
        };

        let mut pipeline = Pipeline {
            profile: p,
            profile_name: profile,
            project_path: project_path.as_ref().to_path_buf(),
            project_structure,
            project_config: config,
            cache_info,
            templates: template_manager,
            cached_contexts: Arc::new(Mutex::new(HashMap::new())),
        };

        let p2 = pipeline.clone();

        // pipeline
        //     .templates
        //     .tera
        //     .register_function("render", p2.create_render_source());

        Ok(pipeline)
    }

    // fn create_render_source(self) -> impl Function { //TODO
    //     Box::new(
    //         move |args: &HashMap<String, Value>| -> tera::Result<Value> {
    //             let val = args
    //                 .get("body")
    //                 .ok_or(tera::Error::msg("missing argument 'body'"))?;
    //             if let Value::String(s) = val {
    //                 let mut doc = Document::try_from(s.as_str()).map_err(tera::Error::msg)?;
    //
    //                 let fstring = args
    //                     .get("format")
    //                     .ok_or(tera::Error::msg("missing argument 'format'"))?
    //                     .to_string();
    //                 let format: Box<dyn Format> = serde_json::from_str(&format!(
    //                     "{{\"{}\": {{}}}}",
    //                     &fstring[1..fstring.len() - 1]
    //                 ))
    //                 .expect("problems!");
    //
    //                 let mut ctx = self.get_render_context(&mut doc, format.borrow()).unwrap();
    //                 let mut renderer = ElementRenderer::new("").unwrap();
    //                 let res = renderer
    //                     .render_doc(
    //                         &mut ctx,
    //                         build_extensions(
    //                             self.profile
    //                                 .render_extensions
    //                                 .get(&fstring)
    //                                 .unwrap_or(&vec![]),
    //                         )
    //                         .map_err(tera::Error::msg)?,
    //                     )
    //                     .map_err(tera::Error::msg)?;
    //                 let val = res.content;
    //                 Ok(Value::String(val.to_string()))
    //             } else {
    //                 Err(tera::Error::msg("invalid type for 'body'"))
    //             }
    //         },
    //     )
    // }

    fn get_render_context<'a>(
        &'a self,
        meta: Metadata,
        format: &'a dyn Format,
    ) -> anyhow::Result<RenderContext<'a>> {
        let mut ctx = Context::default();
        ctx.insert("config", &self.project_config);
        // meta.insert("references", &doc.references);
        ctx.insert("doc_meta", &meta);
        let _ts = &DEFAULT_THEME;
        RenderContext::new(
            &self.templates,
            ctx,
            // &DEFAULT_SYNTAX,
            // &ts.themes["base16-ocean.light"],
            self.project_config.notebook_meta.as_ref().unwrap(),
            format,
            self.profile.parser.settings.clone(),
        )
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
            self.cache_info.reset_entry(
                path.clone().to_str().unwrap().to_string(),
                blake3::hash(val.as_bytes()),
            );
            Ok::<Option<String>, anyhow::Error>(Some(val))
        })?;

        let mut all_errors = Vec::new();

        for format in self.get_formats_or_default().clone() {
            print!("format: {}", style(&format).bold());
            let output = self.process_document(&loaded.doc, format.as_ref());

            match output {
                Err(e) => {
                    all_errors.push(e);
                    println!(" {}", style("error").red());
                }
                Ok(output) => {
                    if let Some(output) = &output {
                        let project = self
                            .cached_contexts
                            .lock()
                            .unwrap()
                            .get(format.name())
                            .ok_or_else(|| anyhow!("Cached context is missing"))?
                            .clone();

                        // let output_raw = output.clone().map(|_c| ());

                        // let project_vec = self.update_cache(
                        //     &item2,
                        //     format.as_ref(),
                        //     &output_raw,
                        //     project.clone(),
                        // );

                        let mut ctx = Generator {
                            root: self.project_path.clone(),
                            project: &from_vec(&project),
                            templates: &self.templates,
                            config: self.project_config.clone(),
                            mode: self.profile.mode,
                            build_dir: self.get_build_path(format.as_ref()),
                            format: format.as_ref(),
                        };
                        ctx.generate_single(output, &item2)?;
                        self.cache_info.update_build_status(
                            item2.doc.path.to_str().unwrap().to_string(),
                            format.name(),
                            true,
                        )?;

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

    // fn update_cache(
    //     &mut self,
    //     item2: &ContentItemDescriptor<()>,
    //     format: &dyn Format,
    //     output: &Document<()>,
    //     mut project: ProjectItemVec,
    // ) -> ProjectItemVec {
    //     let item = project
    //         .iter_mut()
    //         .find(|item| item.path == item2.path)
    //         .expect(&format!("not item found at path: {:?}", item2.path));
    //     item.doc.content = Arc::new(Some(output.clone()));
    //
    //     self.cached_contexts
    //         .lock()
    //         .unwrap()
    //         .insert(format.name().to_string(), project.clone());
    //     project
    // }

    fn doc_from_path(&self, path: PathBuf) -> anyhow::Result<ContentItemDescriptor<()>> {
        let doc_path = path
            .as_path()
            .strip_prefix(self.project_path.as_path().join("content"))?; // TODO: Error handling;

        let path: Vec<String> = vec!["root".to_string()]
            .into_iter()
            .chain(
                doc_path
                    .iter()
                    .map(|d| d.to_str().unwrap().split('.').next().unwrap().to_string()),
            )
            .collect();

        // println!("pp: {:?}", path);

        let path_idx = self
            .project_structure
            .get_path_idx(&path[..])
            .ok_or(anyhow!("Path is invalid"))?;

        // println!("ppi: {:?}", path_idx);

        Ok(ContentItemDescriptor {
            is_section: path.last().unwrap() == "index",
            path,
            path_idx: path_idx.clone(),
            doc: self.project_structure.doc_at_idx(&path_idx[..])?,
        })
    }

    fn get_formats_or_default(&self) -> &Vec<Box<dyn Format>> {
        if self.profile.formats.is_empty() {
            &self.project_config.outputs
        } else {
            &self.profile.formats
        }
    }

    /// Build the whole project (optionally removes existing build)
    pub fn build_all(&mut self, ignore_cache: bool) -> Result<(), anyhow::Error> {
        let build_path = self.project_path.join("build").join(&self.profile_name);

        fs::create_dir_all(&build_path).with_context(|| format!("at {}", build_path.display()))?;

        // let format_folder_names: Vec<&str> = self
        //     .get_formats_or_default()
        //     .iter()
        //     .map(|f| f.name())
        //     .collect();
        // if remove_existing && build_path.exists() {
        //     for entry in
        //         fs::read_dir(&build_path).with_context(|| format!("at {}", build_path.display()))?
        //     {
        //         let entry = entry?;
        //         if entry.path().is_dir()
        //             && format_folder_names
        //                 .iter()
        //                 .any(|f| entry.path().ends_with(f))
        //         {
        //             fs::remove_dir_all(entry.path())
        //                 .with_context(|| format!("at {}", entry.path().display()))?;
        //             fs::create_dir(entry.path())
        //                 .with_context(|| format!("at {}", entry.path().display()))?;
        //         }
        //     }
        // }

        let loaded = self.load_files(ignore_cache)?;

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

        let bar_len: usize = loaded
            .iter()
            .map(|c| c.doc.content.is_some() as usize)
            .sum::<usize>();

        // let bar_len = self.project_structure.len() * 2;
        let sty = ProgressStyle::with_template("{msg:<20} {pos}/{len} {bar:20.cyan/blue}")?;

        for _f in self.get_formats_or_default() {
            let p = ProgressBar::new(bar_len as u64);
            let bar = multi.add(p);
            bar.set_style(sty.clone());

            bars.push(bar);
        }

        let successes = Arc::new(Mutex::new(Vec::new()));

        self.get_formats_or_default()
            .par_iter()
            // .iter()
            .zip(bars.clone())
            .for_each(|(format, bar)| {
                let mut format_errs = Vec::new();

                bar.set_message(format!(
                    "{} {}",
                    style(format.name()).bold(),
                    style("parsing").blue()
                ));
                let output = self.process_all(loaded.clone(), format.as_ref(), bar.clone());

                // let mut errs = Vec::new();
                let output: ProjectItemContentVec = output
                    .into_iter()
                    .filter_map(|item| match item {
                        Ok(item) => Some(item),
                        Err(err) => {
                            format_errs.push(err);
                            None
                        }
                    })
                    .collect();

                let proj = output
                    .iter()
                    .map(|item| {
                        item.map_ref(|doc| {
                            Ok(doc.as_ref().map(|inner| inner.clone().map(|_inner| ())))
                        })
                    })
                    .collect::<anyhow::Result<ProjectItemVec>>()
                    .unwrap();

                // format_errs.append(&mut errs.lock().unwrap())

                let project_full = from_vec(&proj);
                let context = Generator {
                    root: self.project_path.to_path_buf(),
                    project: &project_full,
                    mode: self.profile.mode,
                    templates: &self.templates,
                    config: self.project_config.clone(),
                    format: format.as_ref(),
                    build_dir: self.get_build_path(format.as_ref()),
                };

                self.cached_contexts
                    .lock()
                    .unwrap()
                    .insert(format.name().to_string(), proj);

                bar.set_message(format!(
                    "{} {}",
                    style(format.name()).bold(),
                    style("writing").blue()
                ));
                let res: anyhow::Result<Vec<anyhow::Result<String>>> = context
                    .generate(bar.clone(), &output)
                    .with_context(|| format!("Could not generate {}", format));

                let res = res.unwrap();

                for r in res {
                    match r {
                        Ok(path) => successes
                            .lock()
                            .unwrap()
                            .push((format.name().to_string(), path)),
                        Err(e) => format_errs.push(e),
                    }
                }

                // Move extra files
                let mover = Mover {
                    project_path: self.project_path.to_path_buf(),
                    build_dir: self.get_build_path(format.as_ref()),
                    settings: self.profile.parser.settings.clone(),
                    profile: &self.profile,
                };

                let res = mover.traverse_content(&project_full);
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

        for (format, path) in successes.lock().unwrap().clone().into_iter() {
            self.cache_info
                .update_build_status(path.clone(), &format, true)
                .unwrap()
        }

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
                // eprintln!("backtrace {}", e.backtrace());
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

        let cache_out = serde_json::to_string_pretty(&self.cache_info)?;
        fs::write(
            self.project_path.join(".cache").join("project_info.json"),
            cache_out,
        )?;

        Ok(())
    }

    fn load_files(
        &mut self,
        ignore_cache: bool,
    ) -> anyhow::Result<Vec<ContentItemDescriptor<Option<String>>>> {
        self.project_structure
            .clone()
            .to_vector()
            .into_iter()
            .map(|item| {
                item.map_doc(|doc| {
                    let path = self.project_path.join("content").join(&doc.path);
                    let val = fs::read_to_string(path.as_path())
                        .context(format!("Error loading document {}", path.display()))?;

                    let hash = blake3::hash(val.as_bytes());
                    if ignore_cache || !self.cache_info.matches(doc.path.to_str().unwrap(), hash) {
                        self.cache_info
                            .reset_entry(doc.path.to_str().unwrap().to_string(), hash);
                        Ok(Some(val))
                    } else {
                        Ok(None)
                    }
                })
            })
            .collect::<anyhow::Result<Vec<ContentItemDescriptor<Option<String>>>>>()
    }

    fn process_all(
        &self,
        project: Vec<ContentItemDescriptor<Option<String>>>,
        format: &dyn Format,
        bar: ProgressBar,
    ) -> ProjectItemVecErr {
        let res = project
            .par_iter()
            // .iter()
            .progress_with(bar)
            .map(|i| {
                let res = self.process_document(&i.doc, format).with_context(|| {
                    format!(
                        "Failed to process document – {}",
                        style(format!("content/{}", i.doc.path.display())).italic()
                    )
                });

                res.map(|res| ContentItemDescriptor {
                    is_section: i.is_section,
                    path: i.path.clone(),
                    path_idx: i.path_idx.clone(),
                    doc: DocumentDescriptor {
                        id: i.doc.id.clone(),
                        format: i.doc.format,
                        path: i.doc.path.clone(),
                        content: Arc::new(res),
                    },
                })
            })
            .collect::<Vec<anyhow::Result<ContentItemDescriptor<Option<Document<RenderResult>>>>>>(
            );

        res
    }

    fn process_document(
        &self,
        item: &DocumentDescriptor<Option<String>>,
        format: &dyn Format,
    ) -> anyhow::Result<Option<Document<RenderResult>>> {
        if let Some(content) = item.content.as_ref() {
            let doc = item
                .format
                .loader()
                .load(content, self.profile.mode == Mode::Draft)?;

            match doc {
                None => Ok(None),
                Some(doc) => {
                    if format.no_parse() {
                        Ok(Some(Document {
                            meta: doc.meta,
                            content: "".into(),
                            code_outputs: doc.code_outputs,
                        }))
                    } else if self.profile.mode != Mode::Draft && doc.meta.draft {
                        Ok(Some(doc.map(|_| CowStr::new())))
                    } else if !doc
                        .meta
                        .exclude_outputs
                        .as_ref()
                        .map(|o| o.contains(&format.name().to_string()))
                        .unwrap_or_default()
                    {
                        let processor_ctx = PreprocessorContext {
                            templates: &self.templates,
                            output_format: format,
                            project_root: self.project_path.clone(),
                        };

                        let mut res = self.profile.parser.parse(doc, &processor_ctx)?;

                        // let res = print_err(res)?;

                        let mut ctx = self.get_render_context(res.meta.clone(), format)?;
                        let empty = vec![];
                        let ext = self
                            .profile
                            .render_extensions
                            .get(format.name())
                            .unwrap_or(&empty);

                        // let mut renderer = ElementRenderer::new(build_extensions(ext)?)?;
                        let mut renderer = format.renderer().build(build_extensions(ext)?)?;

                        Ok(Some(renderer.render_doc(&res, &mut ctx)?))
                    } else {
                        Ok(None)
                    }
                }
            }
        } else {
            Ok(None)
        }
    }
}
