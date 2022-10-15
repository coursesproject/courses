use clap::Parser;
use courses::builder::Builder;
use courses::config::Config;
use std::fs::File;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Cli {
    path: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg = Config::from_path(cli.path.clone()).unwrap();

    let mut builder = Builder::new(cli.path.clone(), vec![])?;

    cfg.build(&mut builder)?;

    serde_yaml::to_writer(&File::create(cfg.build_path.join("config.yml"))?, &cfg)?;

    // println!("{:#?}", cfg);
    Ok(())
}
