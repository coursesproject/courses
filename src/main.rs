//! Then what is this??

use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs};

use anyhow::Context;
use clap::{Parser, Subcommand};
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
        name: String,
    },
    Test {},
    Publish {},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { path, mode } => {
            let path = path.unwrap_or(env::current_dir()?);
            let proj = Project::generate_from_directory(path.as_path())?;

            println!("[1/4] ‍💡 Reading project directory...");

            let config_path = path.join("config.yml");
            let config_input = fs::read_to_string(config_path)?;
            let config: ProjectConfig = serde_yaml::from_str(&config_input)
                .context("Error loading project configuration:")?;

            let mut pipeline = Pipeline::new(path.as_path(), mode, config, proj)?;
            pipeline.build_all().context("Build error:")?;

            println!("🌟 Done.");
            Ok(())
        }
        Commands::Serve { path, mode } => {
            let path = path.unwrap_or(env::current_dir()?);
            let proj = Project::generate_from_directory(path.as_path())?;

            println!("[1/4] ‍💡 Reading project directory...");

            let config_path = path.join("config.yml");
            let config_input = fs::read_to_string(config_path)?;
            let config: ProjectConfig = serde_yaml::from_str(&config_input)
                .context("Error loading project configuration:")?;

            let mut pipeline =
                Pipeline::new(path.as_path(), mode.clone(), config.clone(), proj.clone())?;
            pipeline.build_all().context("Build error:")?;

            let p2 = path.as_path().join("content");
            let tp = path.as_path().join("templates");
            let p_build = path.as_path().join("build/html");

            println!("🌟 Done.");

            let (server, controller) = Server::bind(([127, 0, 0, 1], 8000).into())
                .add_mount(config.url_prefix.clone(), p_build)?
                .build()?;

            println!("Server open at: http://localhost:8000{}", config.url_prefix);

            let notify_config = notify::Config::default();
            let mut debouncer: Debouncer<RecommendedWatcher> = new_debouncer_opt(
                Duration::from_millis(10),
                None,
                move |res: DebounceEventResult| match res {
                    Ok(events) => events.iter().for_each(|event| {
                        if let DebouncedEventKind::Any = &event.kind {
                            let p = &event.path;

                            if p.starts_with(path.as_path().join("content")) {
                                // pipeline.build_file(p, &c2, &cf);
                                pipeline
                                    .build_single(p.to_path_buf())
                                    .expect("Serve error:");
                            } else {
                                pipeline.build_all().expect("Serve error");
                            }

                            controller.reload();
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
        Commands::Init { .. } => {
            let options = vec!["Default", "Empty"];
            let mut s = inquire::Select::new("What template set-up would you like?", options);
            s.starting_cursor = 0;
            let res = s.prompt()?;

            setup::setup(res)?;

            Ok(())
        }
        _ => Ok(()),
    }
}
