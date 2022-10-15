use anyhow::Context;
use clap::{Parser, Subcommand};
use courses::builder::Builder;
use courses::config::Config;
use notify::{RecursiveMode, Watcher};
use std::env;
use std::fs::File;
use std::path::PathBuf;

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

fn main() -> anyhow::Result<()> {
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
            let mut watcher = notify::recommended_watcher(move |res| match res {
                Ok(event) => {
                    let mut builder = Builder::new(path.as_path(), vec![]).unwrap();

                    cfg.build(&mut builder).unwrap();
                    println!("event: {:?}", event)
                }
                Err(e) => println!("watch error: {:?}", e),
            })?;

            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            watcher.watch(p2.as_path(), RecursiveMode::Recursive)?;

            loop {}
            Ok(())
        }
        _ => Ok(()),
    }
}
