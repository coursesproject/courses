use crate::config::Config;
use clap::Parser;
use std::fs::File;
use crate::builder::Builder;

mod builder;
mod config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Cli {
    path: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = Config::from_path(cli.path.clone()).unwrap();
    let builder = Builder::new(cli.path.clone())?;

    cfg.build(&builder)?;

    serde_yaml::to_writer(&File::create(cfg.build_path.join("config.yml"))?, &cfg)?;

    // println!("{:#?}", cfg);
    Ok(())
}
