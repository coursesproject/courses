use anyhow::Context;
use clap::{Parser, Subcommand};
use courses::builder::Builder;
use courses::config::Config;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use penguin::Server;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;
use notify::event::ModifyKind;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
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
    let cfg = Config::generate_from_directory(path.as_path()).unwrap();

    match cli.command {
        Commands::Build {} => {
            let mut builder = Builder::new(path.as_path(), vec![])?;

            cfg.build(&mut builder)?;

            serde_yaml::to_writer(&File::create(cfg.build_path.join("config.yml"))?, &cfg)?;

            // println!("{:#?}", cfg);
            Ok(())
        }
        Commands::Serve {} => {
            let p2 = path.as_path().join("content");
            let tp = path.as_path().join("templates");
            let p_build = path.as_path().join("build");

            let mut builder = Builder::new(path.as_path(), vec![]).unwrap();
            cfg.build(&mut builder).unwrap();

            let (server, controller) = Server::bind(([127, 0, 0, 1], 8000).into())
                .add_mount("/", p_build)?
                .build()?;

            let mut watcher = notify::recommended_watcher(move |res| match res {
                Ok(event) => {
                    let event: Event = event;
                    println!("event: {:?}", event);
                    match event.kind {
                        EventKind::Modify(kind) => match kind {
                            ModifyKind::Data(ch) => {
                                println!("Yes");
                                let mut builder = Builder::new(path.as_path(), vec![]).unwrap();

                                cfg.build(&mut builder).unwrap();
                                controller.reload();

                            },
                            _ => {}
                        },
                        _ => {}
                    }


                }
                Err(e) => println!("watch error: {:?}", e),
            })?;

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            watcher.watch(p2.as_path(), RecursiveMode::Recursive)?;
            watcher.watch(tp.as_path(), RecursiveMode::Recursive)?;

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
