use clap::{Parser, Subcommand};
use courses::cfg::Config;
use courses::pipeline::Pipeline;
use notify::event::{AccessKind, ModifyKind};
use notify::{Event, EventKind, INotifyWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{
    new_debouncer_opt, DebounceEventResult, DebouncedEventKind, Debouncer,
};
use penguin::Server;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    path: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Serve {},
    Build {},
    Test {},
    Publish {},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let path = cli.path.unwrap_or(env::current_dir()?);
    let mut cfg = Config::generate_from_directory(path.as_path()).unwrap();

    match cli.command {
        Commands::Build {} => {
            println!("[1/4] ‍💡 Reading project directory...");

            let mut pipeline = Pipeline::new(path.as_path())?;
            let cf = pipeline.build_everything(cfg.clone())?;

            serde_yaml::to_writer(
                &File::create(
                    cfg.clone()
                        .project_path
                        .as_path()
                        .join("build")
                        .join("config.yml"),
                )
                .unwrap(),
                &cf,
            )
            .unwrap();

            println!("🌟 Done.");
            Ok(())
        }
        Commands::Serve {} => {
            let p2 = path.as_path().join("content");
            let tp = path.as_path().join("templates");
            let p_build = path.as_path().join("build/web");

            println!("[1/4] ‍💡 Reading project directory...");
            let config: Config<()> = Config::generate_from_directory(&path)?;
            let c2 = config.clone();

            let mut pipeline = Pipeline::new(path.as_path())?;
            let cf = pipeline.build_everything(config)?;

            serde_yaml::to_writer(
                &File::create(cfg.project_path.join("build").join("config.yml")).unwrap(),
                &cf,
            )
            .unwrap();
            println!("🌟 Done.");

            let (server, controller) = Server::bind(([127, 0, 0, 1], 8000).into())
                .add_mount("/", p_build)?
                .build()?;

            println!("Server open at: http://localhost:8000");

            let config = notify::Config::default();
            let mut debouncer: Debouncer<INotifyWatcher> = new_debouncer_opt(
                Duration::from_millis(10),
                None,
                move |res: DebounceEventResult| match res {
                    Ok(events) => events.iter().for_each(|event| {
                        if let DebouncedEventKind::Any = &event.kind {
                            let p = &event.path;

                            let mut pipeline = Pipeline::new(path.as_path()).unwrap();

                            if p.starts_with(path.as_path().join("content")) {
                                pipeline.build_file(p, &c2, &cf).unwrap();
                            } else {
                                pipeline.build_everything(c2.clone()).unwrap();
                            }

                            controller.reload();
                        }
                    }),
                    Err(errs) => errs.iter().for_each(|e| println!("Error {:?}", e)),
                },
                config,
            )?;

            // let mut watcher = notify::recommended_watcher(move |res| match res {
            //     Ok(event) => {
            //         let event: Event = event;
            //         //println!("event: {:?}", event);
            //         match event.kind {
            //             EventKind::Access(kind) => match kind {
            //                 AccessKind::Close(_) => {}
            //                 _ => {}
            //             },
            //             EventKind::Modify(kind) => match kind {
            //                 ModifyKind::Data(ch) => {
            //                     // let res = pipeline.build_everything(config).unwrap();
            //                     let p = event.paths.first().unwrap();
            //
            //                     let mut pipeline = Pipeline::new(path.as_path()).unwrap();
            //
            //                     if p.starts_with(path.as_path().join("content")) {
            //                         pipeline.build_file(p, &c2, &cf).unwrap();
            //                     } else {
            //                         pipeline.build_everything(c2.clone()).unwrap();
            //                     }
            //
            //                     controller.reload();
            //                 }
            //                 _ => {}
            //             },
            //             _ => {}
            //         }
            //     }
            //     Err(e) => println!("watch error: {:?}", e),
            // })?;

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            debouncer
                .watcher()
                .watch(p2.as_path(), RecursiveMode::Recursive)?;
            debouncer
                .watcher()
                .watch(tp.as_path(), RecursiveMode::Recursive)?;

            // watcher.watch(p2.as_path(), RecursiveMode::Recursive)?;
            // watcher.watch(tp.as_path(), RecursiveMode::Recursive)?;

            // tokio::spawn(async move {
            //     tokio::time::sleep(Duration::from_secs(5)).await;
            //     controller.reload();
            // });

            server.await?;

            Ok(())
        }
        _ => Ok(()),
    }
}
