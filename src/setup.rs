use std::{env, fs};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "resources/bundle/"]
struct Asset;

pub(crate) fn setup(typ: &str) -> anyhow::Result<()> {


    let dir = env::current_dir()?;
    let tp_dir = dir.join("templates");

    fs::create_dir(tp_dir.as_path())?;

    let tp_index_dest = tp_dir.join("index.tera.html");
    let tp_section_dest = tp_dir.join("section.tera.html");
    let tp_nav_dest = tp_dir.join("nav.tera.html");

    match typ {
        "Default" => {
            let tp_index = Asset::get("templates/default/index.tera.html").unwrap();
            let tp_section = Asset::get("templates/default/section.tera.html").unwrap();
            let tp_nav = Asset::get("templates/default/nav.tera.html").unwrap();

            fs::write(tp_index_dest, tp_index.data)?;
            fs::write(tp_section_dest, tp_section.data)?;
            fs::write(tp_nav_dest, tp_nav.data)?;
        }
        "Empty" => {
            let tp_index = Asset::get("templates/empty/index.tera.html").unwrap();
            let tp_section = Asset::get("templates/empty/section.tera.html").unwrap();
            let tp_nav = Asset::get("templates/empty/nav.tera.html").unwrap();

            fs::write(tp_index_dest, tp_index.data)?;
            fs::write(tp_section_dest, tp_section.data)?;
            fs::write(tp_nav_dest, tp_nav.data)?;
        }
        _ => unreachable!()
    }

    let content_dir = dir.join("content");
    fs::create_dir(content_dir.as_path())?;
    let ct_index = Asset::get("index.md").unwrap();
    fs::write(content_dir.join("index.md"), ct_index.data)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_default() {


        for s in vec!["Default", "Empty"] {
            let mut tmpdir = tempfile::tempdir().unwrap();
            env::set_current_dir(tmpdir.path()).expect("Could not set working directory");

            setup(s).expect("Setup failed");

            let index_path = tmpdir.path().join("content/index.md");
            let tp_index_path = tmpdir.path().join("templates/index.tera.html");
            let tp_section_path = tmpdir.path().join("templates/section.tera.html");
            let tp_nav_path = tmpdir.path().join("templates/nav.tera.html");
            assert!(index_path.is_file(), "Missing index.md");
            assert!(tp_index_path.is_file(), "Missing index.tera.html");
            assert!(tp_section_path.is_file(), "Missing section.tera.html");
            assert!(tp_nav_path.is_file(), "Missing nav.tera.html");
        }

    }
}