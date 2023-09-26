use anyhow::anyhow;
use cdoc::templates::TemplateManager;
use clap::Parser;
use include_dir::{include_dir, Dir};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,
}

#[allow(unused)]
static TEMPLATE_FILES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/resources/config");

fn ensure_config_created(config_path: &PathBuf) -> anyhow::Result<()> {
    if !config_path.exists() {
        fs::create_dir_all(config_path)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let _args = Args::parse();
    let config_path = dirs::home_dir()
        .ok_or(anyhow!("Home directory not found"))?
        .join(".cdoc");

    ensure_config_created(&config_path)?;

    let _manager = TemplateManager::from_path(
        config_path.join("templates"),
        config_path.join("filters"),
        true,
    )?;

    Ok(())
}
