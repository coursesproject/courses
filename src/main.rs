//! Then what is this??

use std::fs::create_dir;
use std::path::{Path, PathBuf};
use std::time::Duration;
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

use courses::pipeline::Pipeline;
use courses::project::config::ProjectConfig;
use courses::project::Project;

mod setup;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Serve {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(short, long, default_value = "dev")]
        mode: String,
    },
    Build {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(short, long, default_value = "release")]
        mode: String,
    },
    Init {
        name: Option<String>,
        #[arg(short, long)]
        repository: Option<String>,
    },
    Create {},
    Test {},
    Publish {},
}

async fn cli_run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { path, mode } => {
            let current_time = std::time::Instant::now();
            let path = path.unwrap_or(env::current_dir()?);

            print!("Configuring project...");
            let proj = Project::generate_from_directory(path.as_path())?;
            println!(" {}", style("done").green());

            let config_path = path.join("config.yml");
            let config_input = fs::read_to_string(config_path)?;
            let config: ProjectConfig = serde_yaml::from_str(&config_input)
                .context("Could not load project configuration")?;

            let mut pipeline = Pipeline::new(path.as_path(), mode, config, proj)?;
            pipeline.build_all(true)?;

            println!("🌟 Done ({} ms)", current_time.elapsed().as_millis());
            Ok(())
        }
        Commands::Serve { path, mode } => {
            let path = path.unwrap_or(env::current_dir()?);
            let mut absolute_path = env::current_dir()?;
            absolute_path.push(path.as_path());

            print!("Configuring project...");
            let proj = Project::generate_from_directory(path.as_path())?;
            println!(" {}", style("done").green());

            let config_path = path.join("config.yml");
            let config_input = fs::read_to_string(config_path)?;
            let config: ProjectConfig = serde_yaml::from_str(&config_input)
                .context("Could not load project configuration")?;

            let mut pipeline = Pipeline::new(
                absolute_path.as_path(),
                mode.clone(),
                config.clone(),
                proj.clone(),
            )?;

            let res = pipeline.build_all(true).context("Build error:");
            err_print(res);

            let p2 = path.as_path().join("content");
            let tp = path.as_path().join("templates");
            let p_build = path.as_path().join("build/html");

            let (server, controller) = Server::bind(([127, 0, 0, 1], 8000).into())
                .add_mount(config.url_prefix.clone(), p_build)?
                .build()?;

            print!("\n\n");

            println!("Server open at: http://localhost:8000{}", config.url_prefix);

            let notify_config = notify::Config::default();

            let mut debouncer: Debouncer<RecommendedWatcher> = new_debouncer_opt(
                Duration::from_millis(20),
                None,
                move |res: DebounceEventResult| match res {
                    Ok(events) => events.iter().for_each(|event| {
                        if let DebouncedEventKind::Any = &event.kind {
                            let full_path = &event.path;
                            let p = full_path.strip_prefix(absolute_path.as_path()).unwrap();
                            println!();

                            if p.starts_with(Path::new("content")) {
                                // pipeline.build_file(p, &c2, &cf);
                                let res = pipeline.build_single(full_path.to_path_buf());
                                err_print(res);
                            } else {
                                if p.starts_with(Path::new("templates").join("shortcodes")) {
                                    let res = pipeline.reload_shortcode_tera();
                                    err_print(res);
                                    println!("{}", style("reloaded shortcode templates").green());
                                } else if p.starts_with(Path::new("templates").join("builtins")) {
                                    let res = pipeline.reload_builtins_tera();
                                    err_print(res);
                                    println!("{}", style("reloaded builtin templates").green());
                                }
                                let res = pipeline.reload_base_tera();
                                println!("{}", style("Reloaded page templates").green());
                                err_print(res);

                                let res = pipeline.build_all(false);
                                err_print(res);
                            }

                            controller.reload();
                            println!();
                            println!("Page reloaded");
                            println!("Server open at: http://localhost:8000{}", config.url_prefix);
                        }
                    }),
                    Err(errs) => errs.iter().for_each(|e| println!("Error {:?}", e)),
                },
                notify_config,
            )?;

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            debouncer
                .watcher()
                .watch(p2.as_path(), RecursiveMode::Recursive)?;
            debouncer
                .watcher()
                .watch(tp.as_path(), RecursiveMode::Recursive)?;

            server.await?;

            Ok(())
        }
        Commands::Init { name, repository } => {
            println!("{}", style("Project initialisation").bold().blue());

            let repository = repository.map(Ok::<String, InquireError>).unwrap_or_else(|| {
                let options = vec!["Default", "Empty"];
                let values = vec!["https://github.com/coursesproject/courses-template-default/archive/main.zip", ""];

                let mut s: Select<&str> = Select::new("Select a project template:", options);

                s.starting_cursor = 0;
                Ok(values[s.raw_prompt()?.index].to_string())
            })?;

            let dir = name
                .map(|n| {
                    let dir = env::current_dir()?.join(n);
                    create_dir(dir.as_path())?;
                    Ok(dir)
                })
                .unwrap_or_else(env::current_dir)?;

            print!("Template downloading ...");

            setup::setup(dir, repository)?;

            println!("{}", style("Success").green().bold());

            Ok(())
        }
        Commands::Create {} => {
            let res = inquire::Text::new("Document path: ").prompt()?;
            let doc_path = Path::new(&res);

            let ext = doc_path
                .extension()
                .ok_or(anyhow!(
                    "Document path must have an extension (.md or .ipynb)"
                ))?
                .to_str()
                .unwrap();

            let title = inquire::Text::new("Enter document title: ").prompt()?;

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

            let output_formats = vec!["notebook", "html", "info", "latex"];
            let output_ans = inquire::MultiSelect::new("Select output formats:", output_formats)
                .with_default(&[2])
                .prompt()?;

            let mut doc_meta: LinkedHashMap<String, serde_yaml::Value> = LinkedHashMap::new();
            doc_meta.insert("title".to_string(), serde_yaml::Value::from(title));
            doc_option_ans.into_iter().for_each(|o| {
                doc_meta.insert(o.to_string(), serde_yaml::Value::from(true));
            });
            doc_meta.insert("outputs".to_string(), serde_yaml::Value::from(output_ans));

            let doc_meta_string = serde_yaml::to_string(&doc_meta)?;
            let content_path = env::current_dir()?.join("content");
            let full_path = content_path.join(doc_path);

            let doc_string = match ext {
                "md" => Ok(format!("---\n{doc_meta_string}\n---\n")),
                "ipynb" => {
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

fn err_print(res: anyhow::Result<()>) {
    match res {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {:?}", style("Error:").red().bold(), e);
            e.chain().skip(1).for_each(|cause| {
                eprintln!(
                    " {} {}\n{}",
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
async fn main() {
    err_print(cli_run().await)
}
