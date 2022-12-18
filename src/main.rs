//! Then what is this??

use std::env;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

use courses::pipeline::Pipeline;
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

            let mut pipeline = Pipeline::new(path.as_path(), mode, proj)?;
            pipeline.build_all().context("Build error:")?;

            println!("🌟 Done.");
            Ok(())
        }
        // Commands::Serve { path, mode } => {
        //     let path = path.unwrap_or(env::current_dir()?);
        //     let cfg = Project::generate_from_directory(path.as_path()).unwrap();
        //
        //     let p2 = path.as_path().join("content");
        //     let tp = path.as_path().join("templates");
        //     let p_build = path.as_path().join("build/web");
        //
        //     println!("[1/4] ‍💡 Reading project directory...");
        //     let config: Project<()> = Project::generate_from_directory(&path)?;
        //     let c2 = config.clone();
        //
        //     let mut pipeline = Pipeline::new(path.as_path(), mode.clone())?;
        //     let cf = pipeline.build_everything(config)?;
        //
        //     serde_yaml::to_writer(
        //         &File::create(cfg.project_path.join("build").join("config.yml")).unwrap(),
        //         &cf,
        //     )
        //     .unwrap();
        //     println!("🌟 Done.");
        //
        //     let config_path = path.as_path().join("config.yml");
        //     let config_reader = BufReader::new(File::open(config_path)?);
        //     let project_config: ProjectConfig = serde_yaml::from_reader(config_reader)?;
        //
        //     let (server, controller) = Server::bind(([127, 0, 0, 1], 8000).into())
        //         .add_mount(project_config.url_prefix.clone(), p_build)?
        //         .build()?;
        //
        //     println!(
        //         "Server open at: http://localhost:8000{}",
        //         project_config.url_prefix
        //     );
        //
        //     let config = notify::Config::default();
        //     let mut debouncer: Debouncer<RecommendedWatcher> = new_debouncer_opt(
        //         Duration::from_millis(10),
        //         None,
        //         move |res: DebounceEventResult| match res {
        //             Ok(events) => events.iter().for_each(|event| {
        //                 if let DebouncedEventKind::Any = &event.kind {
        //                     let p = &event.path;
        //
        //                     let mut pipeline = Pipeline::new(path.as_path(), mode.clone()).unwrap();
        //
        //                     if p.starts_with(path.as_path().join("content")) {
        //                         pipeline.build_file(p, &c2, &cf);
        //                     } else {
        //                         pipeline.build_everything(c2.clone()).unwrap();
        //                     }
        //
        //                     controller.reload();
        //                 }
        //             }),
        //             Err(errs) => errs.iter().for_each(|e| println!("Error {:?}", e)),
        //         },
        //         config,
        //     )?;
        //
        //     // Add a path to be watched. All files and directories at that path and
        //     // below will be monitored for changes.
        //     debouncer
        //         .watcher()
        //         .watch(p2.as_path(), RecursiveMode::Recursive)?;
        //     debouncer
        //         .watcher()
        //         .watch(tp.as_path(), RecursiveMode::Recursive)?;
        //
        //     server.await?;
        //
        //     Ok(())
        // }
        // Commands::Init { .. } => {
        //     let options = vec!["Default", "Empty"];
        //     let mut s = inquire::Select::new("What template set-up would you like?", options);
        //     s.starting_cursor = 0;
        //     let res = s.prompt()?;
        //
        //     setup::setup(res)?;
        //
        //     Ok(())
        // }
        _ => Ok(()),
    }
}
