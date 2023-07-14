use anyhow::Context;
use std::fs;
use std::path::PathBuf;

use crate::project::config::{Mode, Profile};
use crate::project::{ContentItem, ContentResultS};
use cdoc::parser::ParserSettings;
use cdoc::parsers::split::parse_code_string;
use cdoc::parsers::split_types::Output;

/// Type that implements copying resource files from within a project's content folder to
/// the build output folder.
pub struct Mover<'a> {
    /// Project root path.
    pub project_path: PathBuf,
    /// Build directory (relative to [project_path]).
    pub build_dir: PathBuf,
    /// Used to determine whether to include solutions in python script files.
    pub settings: ParserSettings,
    pub profile: &'a Profile,
}

impl Mover<'_> {
    fn create_build_path(
        content_path: PathBuf,
        build_path: PathBuf,
        entry_path: PathBuf,
    ) -> anyhow::Result<PathBuf> {
        let base_path = entry_path.strip_prefix(content_path)?;
        Ok(build_path.join(base_path))
    }

    fn content_path(&self) -> PathBuf {
        self.project_path.join("content")
    }

    fn get_children_dirs(&self, children: &[ContentResultS]) -> Vec<String> {
        children
            .iter()
            .filter_map(|c| match c {
                ContentItem::Section { section_id, .. } => Some(section_id.to_string()),
                _ => None,
            })
            .collect()
    }

    pub fn traverse_content(&self, item: &ContentResultS) -> anyhow::Result<()> {
        if let ContentItem::Section {
            section_id,
            section_path,
            children,
            doc,
            ..
        } = item
        {
            if self.profile.mode == Mode::Draft
                || doc
                    .content
                    .as_ref()
                    .as_ref()
                    .is_some_and(|c| !c.metadata.draft)
            {
                let dir_path = self.content_path().join(section_path);
                // println!("dir {}", dir_path.display());
                for child in children {
                    self.traverse_content(child)?
                }
                let exclusions = self.get_children_dirs(children);
                if section_id != "root" {
                    self.traverse_dir(dir_path.clone(), exclusions)
                        .with_context(|| {
                            format!("Resource copy from directory {} failed", dir_path.display())
                        })?;
                }
            }
        }
        Ok(())
    }

    /// Does the actual copying and calls itself recursively for subdirectories.
    pub fn traverse_dir(&self, path: PathBuf, exclude: Vec<String>) -> anyhow::Result<()> {
        let content_path = self.project_path.join("content");
        let exclude: Vec<PathBuf> = exclude.iter().map(|p| path.join(p)).collect();

        for entry in fs::read_dir(path.as_path())
            .with_context(|| format!("directory not found at {}", path.display()))?
        {
            let entry = entry?;
            let entry_path = entry.path();

            let metadata = fs::metadata(entry.path())?;

            if metadata.is_file() {
                let dest = Mover::create_build_path(
                    content_path.to_path_buf(),
                    self.build_dir.to_path_buf(),
                    entry_path.to_path_buf(),
                )?;
                if let Some(ext_os) = entry_path.as_path().extension() {
                    let ext = ext_os.to_str().unwrap();

                    match ext {
                        "md" | "ipynb" => {}
                        "py" => {
                            let input =
                                fs::read_to_string(entry_path.as_path()).with_context(|| {
                                    format!(
                                        "failed to read python file at {}",
                                        entry_path.as_path().display()
                                    )
                                })?;
                            let parsed = parse_code_string(&input)?;
                            let output = parsed.write_string(self.settings.solutions);

                            // let mut file = fs::OpenOptions::new().write(true).create(true).append(false).open(section_build_path)?;
                            // file.write_all(output.as_bytes())?;
                            fs::create_dir_all(dest.as_path())?;

                            fs::write(dest, output).with_context(|| {
                                format!(
                                    "failed to write python file at {}",
                                    entry_path.as_path().display()
                                )
                            })?;
                        }
                        _ => {
                            if !entry_path.is_dir() {
                                fs::create_dir_all(dest.as_path().parent().unwrap())?;
                                fs::copy(&entry_path, dest).with_context(|| {
                                    format!(
                                        "failed to copy file to build folder at {}",
                                        entry_path.as_path().display()
                                    )
                                })?;
                            }
                        }
                    }
                }
            } else if !exclude.contains(&entry_path) {
                self.traverse_dir(entry_path, vec![])?;
            }
        }

        Ok(())
    }
}
