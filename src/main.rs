//! Then what is this??

use std::fs::create_dir;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};
use std::{env, fs};

use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use console::style;

use inquire::{InquireError, Select};
use linked_hash_map::LinkedHashMap;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{
    new_debouncer_opt, DebounceEventResult, DebouncedEventKind, Debouncer,
};
use penguin::Server;
use semver::{Version, VersionReq};

use courses::pipeline::Pipeline;
use courses::project::config::{ProjectConfig, ProjectConfigDummy};
use courses::project::{configure_project, from_vec, ContentItem};

use courses::built_info;

mod setup;

#[derive(Parser, Debug)]
#[command(name = "Courses")]
#[command(author = "Anton Mølbjerg Eskildsen <antonmeskildsen@me.com>")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build and serve the project locally. Recompiles whenever a file change is detected.
    Serve {
        /// Optional path to the project root directory (that contains config.yml).
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Build profile (defined in config.yml).
        #[arg(short = 'o', long, default_value = "draft")]
        profile: String,
    },
    /// Build the project with the given profile.
    Build {
        /// Optional path to the project root directory (that contains config.yml).
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Build profile (defined in config.yml).
        #[arg(short = 'o', long, default_value = "release")]
        profile: String,
    },
    /// Create a new courses project.
    Init {
        name: Option<String>,
        /// Template repository to use for the project.
        #[arg(short, long)]
        repository: Option<String>,
    },
    Create {},
    Test {},
    /// Run a project script (as defined in config.yml)
    Run {
        /// Script name.
        script: String,
        /// Optional path to the project root directory (that contains config.yml).
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Arguments passed to the script
        #[arg(last = true)]
        run_args: Vec<String>,
    },
    Update {
        /// Optional path to the project root directory (that contains config.yml).
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(short, long)]
        version: Option<VersionReq>,
    },
}

fn path_with_default(path: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let mut p = env::current_dir()?;
    Ok(match path {
        None => p,
        Some(path) => {
            p.push(path.as_path());
            p
        }
    })
}

fn init_config(path: &Path, allow_incompatible: bool) -> anyhow::Result<ProjectConfig> {
    print!("Reading config...");

    let config_path = path.join("config.yml");
    let config_input = fs::read_to_string(config_path)?;
    if !allow_incompatible {
        let dummy: ProjectConfigDummy = serde_yaml::from_str(&config_input)?;
        let pkg_version = Version::parse(built_info::PKG_VERSION)?;
        if !dummy.courses.version.matches(&pkg_version) {
            return Err(anyhow!(format!(
            "Incompatible courses version ({}) for project requiring {}. Please update courses with `courses update`",
            pkg_version, dummy.courses.version
        )));
        }
    }

    let config: ProjectConfig =
        serde_yaml::from_str(&config_input).context("Could not load project configuration")?;

    println!(" {}", style("done").green());
    Ok(config)
}

fn init_project_structure(path: &Path) -> anyhow::Result<ContentItem<()>> {
    print!("Configuring project structure...");
    let proj = configure_project(path.join("content"));
    println!(" {}", style("done").green());
    proj
}

fn init_pipeline(
    absolute_path: &Path,
    profile: String,
    config: ProjectConfig,
    structure: ContentItem<()>,
) -> anyhow::Result<Pipeline> {
    Pipeline::new(absolute_path, profile, config, structure)
}

fn init_and_build(path: Option<PathBuf>, profile: String) -> anyhow::Result<(Pipeline, PathBuf)> {
    let path = path_with_default(path)?;
    let config = init_config(&path, false)?;
    let structure = init_project_structure(&path)?;

    // let mut absolute_path = env::current_dir()?;
    // absolute_path.push(path.as_path());

    let pipeline = init_pipeline(&path, profile, config, structure)?;

    Ok((pipeline, path))
}

async fn cli_run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Update { path, version } => {
            let mut cmd = Command::new("cargo");
            if let Some(version) = version {
                cmd.arg("install")
                    .arg("courses")
                    .arg("--version")
                    .arg(version.to_string());
            } else {
                let path = path_with_default(path)?;
                let config = init_config(&path, true)?;
                cmd.arg("install")
                    .arg("courses")
                    .arg("--version")
                    .arg(config.courses.version.to_string());
            }

            let mut res = cmd.spawn()?;
            let _status = res.wait()?;
            Ok(())
        }
        Commands::Build { path, profile } => {
            let current_time = SystemTime::now();
            let (mut pipeline, _) = init_and_build(path, profile)?;
            pipeline.build_all(true)?;

            println!("🌟 Done ({} ms)", current_time.elapsed()?.as_millis());
            Ok(())
        }
        Commands::Run {
            path,
            script,
            run_args,
        } => {
            let path = path_with_default(path)?;
            let config = init_config(&path, false)?;
            let s = config.scripts.get(&script).unwrap();
            let mut cmd = Command::new("bash");

            cmd.arg("-c").arg(format!("{} {}", s, run_args.join(" ")));

            println!("running {:?}", cmd);

            cmd.stdin(Stdio::inherit()).status()?;

            // assert!(output.status.success());
            Ok(())
        }
        Commands::Serve { path, profile } => {
            // Used for measuring build time
            let current_time = SystemTime::now();

            // Initialize pipeline and build the project
            let (mut pipeline, absolute_path) = init_and_build(path.clone(), profile.clone())?;
            let stuff: ContentItem<()> = from_vec(&pipeline.project_structure.clone().to_vector());
            let config_out = serde_json::to_string(&stuff)?;

            fs::write(absolute_path.join("project.json"), config_out).unwrap();
            let res = pipeline.build_all(true).context("Build error:");
            err_print(res);
            println!("🌟 Done ({} ms)", current_time.elapsed()?.as_millis());

            // Initialize local server
            let path = path_with_default(path)?;
            let p_build = path.as_path().join("build").join(&profile).join("html");
            let (server, controller) = Server::bind(([127, 0, 0, 1], 8000).into())
                .add_mount(pipeline.project_config.url_prefix.clone(), p_build)?
                .build()?;

            print!("\n\n");

            println!(
                "Server open at: http://localhost:8000{}",
                pipeline.project_config.url_prefix
            );

            let notify_config = notify::Config::default();

            // Notifier that watches for file changes.
            let mut debouncer: Debouncer<RecommendedWatcher> = new_debouncer_opt(
                Duration::from_millis(100),
                None,
                move |res: DebounceEventResult| match res {
                    Ok(events) => events.iter().for_each(|event| {
                        // Only listen for this specific kind of event that is emitted for
                        // file changes on any operating system.
                        if let DebouncedEventKind::Any = &event.kind {
                            let full_path = &event.path;
                            let p = full_path.strip_prefix(absolute_path.as_path()).unwrap();

                            // Ensure that temporary files (that include ~) created by some editors are skipped
                            if let Some(ext) = p.extension() {
                                if !ext.to_str().unwrap().contains('~') {
                                    let current_time = SystemTime::now();
                                    // Determine whether change was in content or template
                                    if p.starts_with(Path::new("content")) {
                                        // Content change only results in rebuilding the changed file itself.
                                        let res = pipeline.build_single(full_path.to_path_buf());
                                        err_print(res);
                                    } else if p.starts_with(Path::new("templates")) {
                                        // Template change requires reloading templates and rebuilding the whole
                                        // project.
                                        let res = pipeline.reload_templates();
                                        err_print(res);
                                        println!("{}", style("reloaded templates").green());
                                        let res = pipeline.build_all(false);
                                        err_print(res);
                                    } else if p.starts_with(Path::new("scripts")) {
                                        let res = pipeline.build_all(false);
                                        err_print(res);
                                    }

                                    // Reload the webpage to show the updated content.
                                    controller.reload();
                                    println!();
                                    println!(
                                        "🌟 Done ({} ms)",
                                        current_time.elapsed().unwrap().as_millis()
                                    );

                                    println!(
                                        "{}http://localhost:8000{}",
                                        style("Server open at: ").italic(),
                                        pipeline.project_config.url_prefix
                                    );
                                }
                            }
                        }
                    }),
                    Err(errs) => errs.iter().for_each(|e| println!("Error {:?}", e)),
                },
                notify_config,
            )?;

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            debouncer.watcher().watch(
                path.as_path().join("content").as_path(),
                RecursiveMode::Recursive,
            )?;
            debouncer.watcher().watch(
                path.as_path().join("templates").as_path(),
                RecursiveMode::Recursive,
            )?;
            debouncer.watcher().watch(
                path.as_path().join("scripts").as_path(),
                RecursiveMode::Recursive,
            )?;

            server.await?;

            Ok(())
        }
        // Create a new project.
        Commands::Init { name, repository } => {
            println!("{}", style("Project initialisation").bold().blue());

            // Select template repository (currently only supports Default)
            let repository = repository.map(Ok::<String, InquireError>).unwrap_or_else(|| {
                let options = vec!["Default", "Empty"];
                let values = ["https://github.com/coursesproject/courses-template-default/archive/main.zip", ""];

                let mut s: Select<&str> = Select::new("Select a project template:", options);

                s.starting_cursor = 0;
                Ok(values[s.raw_prompt()?.index].to_string())
            })?;

            // Prepare directory for project
            let dir = name
                .map(|n| {
                    let dir = env::current_dir()?.join(n);
                    create_dir(dir.as_path())?;
                    Ok(dir)
                })
                .unwrap_or_else(env::current_dir)?;

            print!("Template downloading ...");

            // Download and unpack the template in the new directory
            setup::setup(dir, repository)?;

            println!("{}", style("Success").green().bold());

            Ok(())
        }
        // Wizard for creating a new content file
        Commands::Create {} => {
            // Ask for path
            let res = inquire::Text::new("Document path: ").prompt()?;
            let doc_path = Path::new(&res);

            let ext = doc_path
                .extension()
                .ok_or(anyhow!(
                    "Document path must have an extension (.md or .ipynb)"
                ))?
                .to_str()
                .unwrap();

            // Ask for doc title
            let title = inquire::Text::new("Enter document title: ").prompt()?;

            // Document options (metadata)
            let doc_options = vec![
                "exercises",
                "notebook_output",
                "code_solutions",
                "cell_outputs",
                "interactive",
                "editable",
            ];

            let doc_option_ans =
                inquire::MultiSelect::new("Select document options: ", doc_options)
                    .with_default(&[0, 3])
                    .prompt()?;

            // Determine output formats
            let output_formats = vec!["notebook", "html", "info", "latex"];
            let output_ans = inquire::MultiSelect::new("Select output formats:", output_formats)
                .with_default(&[2])
                .prompt()?;

            // Construct Map with chosen options
            let mut doc_meta: LinkedHashMap<String, serde_yaml::Value> = LinkedHashMap::new();
            doc_meta.insert("title".to_string(), serde_yaml::Value::from(title));
            doc_option_ans.into_iter().for_each(|o| {
                doc_meta.insert(o.to_string(), serde_yaml::Value::from(true));
            });
            doc_meta.insert("outputs".to_string(), serde_yaml::Value::from(output_ans));
            // Write options to yaml string
            let doc_meta_string = serde_yaml::to_string(&doc_meta)?;

            let content_path = env::current_dir()?.join("content");
            let full_path = content_path.join(doc_path);

            // Determine creation process based on file extension.
            let doc_string = match ext {
                "md" => Ok(format!("---\n{doc_meta_string}\n---\n")), // markdown inserts options at the top
                "ipynb" => {
                    // Notebooks require a template to work properly. Options are inserted in a raw cell.
                    let lines: Vec<String> =
                        doc_meta_string.lines().map(|l| format!("{l}\n")).collect();

                    let lines_str = serde_json::to_string_pretty(&lines)?;
                    let raw_upper = r#"
                    {
                        "cells": [
                            {
                                "cell_type": "raw",
                                "metadata": {
                                    "collapsed": true
                                },
                                "source":"#;
                    let raw_lower = r#"
                            },
                            {
                                "cell_type": "code",
                                "execution_count": null,
                                "metadata": {
                                    "collapsed": true
                                },
                                "outputs": [],
                                "source": []
                            }
                        ],
                        "metadata": {
                            "kernelspec": {
                                "display_name": "Python 3",
                                "language": "python",
                                "name": "python3"
                            },
                            "language_info": {
                                "codemirror_mode": {
                                    "name": "ipython",
                                    "version": 2
                                },
                                "file_extension": ".py",
                                "mimetype": "text/x-python",
                                "name": "python",
                                "nbconvert_exporter": "python",
                                "pygments_lexer": "ipython2",
                                "version": "2.7.6"
                            }
                        },
                        "nbformat": 4,
                        "nbformat_minor": 0
                    }
                    "#;
                    Ok(format!("{raw_upper}\n{lines_str}\n{raw_lower}"))
                }
                _ => Err(anyhow!("Invalid extension. Must be one of: .md, .ipynb")),
            }?;

            fs::write(full_path, doc_string)?;
            println!("Done!");

            Ok(())
        }
        _ => Ok(()),
    }
}

// Print errors in red if present.
fn err_print(res: anyhow::Result<()>) {
    match res {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {:?}", style("Error:").red().bold(), e);
            e.chain().skip(1).for_each(|cause| {
                eprintln!(
                    " {} {} bt: {}",
                    style("caused by:").bold(),
                    cause,
                    e.backtrace()
                )
            });
            // eprintln!("{}", e.backtrace());
        }
    }
}

#[tokio::main]
// #[pollster::main]
async fn main() {
    err_print(cli_run().await)
}
