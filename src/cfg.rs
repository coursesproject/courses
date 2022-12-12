//! Types for describing project configurations.

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};

/// This is a custom map trait meant for configurations. The structure is preserved and each document
/// is transformed.
pub trait Transform<T, I, O> {
    /// Like map but for a configuration.
    fn transform<F>(&self, f: &F) -> T
    where
        F: Fn(&DocumentSpec<I>) -> O;
}

/// Convenience trait for a map-function that also has access to the possible parents of a document.
pub trait TransformParents<T, I, O> {
    /// The inner function receives the parents which are omitted if they don't exist.
    fn transform_parents<F>(&self, f: &F) -> T
    where
        F: Fn(&DocumentSpec<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O;
}

impl<D> IntoIterator for Config<D>
where
    D: Clone,
{
    type Item = ConfigItem<D>;
    type IntoIter = ConfigIterator<D>;

    fn into_iter(self) -> Self::IntoIter {
        ConfigIterator {
            part_pos: 0,
            chapter_pos: 0,
            doc_pos: 0,
            config: self,
        }
    }
}

/// Iterates a Config.
pub struct ConfigIterator<D> {
    part_pos: usize,
    chapter_pos: usize,
    doc_pos: usize,
    config: Config<D>,
}

/// Contains necessary information for reconstructing a Config from an iterator.
pub struct ConfigItem<D> {
    pub part_id: Option<String>,
    pub chapter_id: Option<String>,
    pub part_idx: Option<usize>,
    pub chapter_idx: Option<usize>,
    pub doc: DocumentSpec<D>,
    pub files: Option<Vec<PathBuf>>, // Temporary solution for carrying file info
}

impl<D> ConfigItem<D> {
    fn new(
        part_id: Option<String>,
        chapter_id: Option<String>,
        part_idx: Option<usize>,
        chapter_idx: Option<usize>,
        doc: DocumentSpec<D>,
        files: Option<Vec<PathBuf>>,
    ) -> Self {
        ConfigItem {
            part_id,
            chapter_id,
            part_idx,
            chapter_idx,
            doc,
            files,
        }
    }

    /// Perform operation on the inner document, then return the result wrapped in a ConfigItem.
    pub fn map<O, F>(self, f: F) -> anyhow::Result<ConfigItem<O>>
    where
        F: Fn(&D) -> anyhow::Result<O>,
    {
        let doc = DocumentSpec {
            id: self.doc.id,
            format: self.doc.format,
            path: self.doc.path,
            content: Arc::new(f(self.doc.content.as_ref())?),
        };
        Ok(ConfigItem::new(
            self.part_id,
            self.chapter_id,
            self.part_idx,
            self.chapter_idx,
            doc,
            self.files,
        ))
    }

    /// Perform operation on the whole DocumentSpec.
    pub fn map_doc<O, F, E>(self, f: F) -> Result<ConfigItem<O>, E>
    where
        F: Fn(DocumentSpec<D>) -> Result<O, E>,
    {
        let doc = DocumentSpec {
            id: self.doc.id.clone(),
            format: self.doc.format.clone(),
            path: self.doc.path.clone(),
            content: Arc::new(f(self.doc)?),
        };
        Ok(ConfigItem::new(
            self.part_id,
            self.chapter_id,
            self.part_idx,
            self.chapter_idx,
            doc,
            self.files,
        ))
    }

    // pub fn get_chapter<T>(&self, config: Config<T>) -> Option<Chapter<T>> {
    //     config.content[self.par]
    // }
}

/// Collect iterator of ConfigItem into Config (tree structure).
impl<D: Clone + Default> FromIterator<ConfigItem<D>> for Config<D> {
    fn from_iter<T: IntoIterator<Item = ConfigItem<D>>>(iter: T) -> Self {
        // let mut index = it.next().unwrap().doc;
        let mut index: DocumentSpec<D> = DocumentSpec {
            id: "".to_string(),
            format: Format::Markdown,
            path: Default::default(),
            content: Arc::new(D::default()),
        };

        let mut parts: Vec<Part<D>> = vec![];

        let mut last_chapter = 0;

        for item in iter {
            match item.part_idx.unwrap() {
                0 => index = item.doc,
                _part_idx => {
                    let part_id = item.part_id.unwrap();
                    match item.chapter_idx.unwrap() {
                        0 => {
                            last_chapter = 0;
                            parts.push(Part {
                                id: part_id,
                                index: item.doc,
                                chapters: vec![],
                            })
                        }
                        chapter_idx => {
                            let chapter_id = item.chapter_id.unwrap();

                            if last_chapter == chapter_idx {
                                parts
                                    .last_mut()
                                    .unwrap()
                                    .chapters
                                    .last_mut()
                                    .unwrap()
                                    .documents
                                    .push(item.doc);
                            } else {
                                parts.last_mut().unwrap().chapters.push(Chapter {
                                    id: chapter_id,
                                    index: item.doc,
                                    documents: vec![],
                                    files: item.files.expect("No files"),
                                });
                                last_chapter = chapter_idx;
                            }
                        }
                    }
                }
            }
        }

        Config {
            project_path: Default::default(),
            index,
            content: parts,
        }
    }
}

impl<D> Iterator for ConfigIterator<D>
where
    D: Clone,
{
    type Item = ConfigItem<D>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.part_pos {
            0 => {
                // Config index
                self.part_pos += 1;
                Some(ConfigItem::new(
                    None,
                    None,
                    Some(0),
                    None,
                    self.config.index.clone(),
                    None,
                ))
            }
            part_idx if part_idx <= self.config.content.len() => {
                let part = &self.config.content[part_idx - 1];

                let current_chapter_pos = self.chapter_pos;

                match current_chapter_pos {
                    0 => {
                        // Part index
                        if part.chapters.is_empty() {
                            self.part_pos += 1;
                        } else {
                            self.chapter_pos += 1;
                        }
                        Some(ConfigItem::new(
                            Some(part.id.clone()),
                            None,
                            Some(part_idx),
                            Some(0),
                            part.index.clone(),
                            None,
                        ))
                    }

                    chapter_idx => {
                        let chapter = &part.chapters[chapter_idx - 1];

                        let current_doc_pos = self.doc_pos;

                        if current_doc_pos >= chapter.documents.len() {
                            if current_chapter_pos >= part.chapters.len() {
                                self.part_pos += 1;
                                self.chapter_pos = 0;
                            } else {
                                self.chapter_pos += 1;
                            }
                            self.doc_pos = 0;
                        } else {
                            self.doc_pos += 1;
                        }

                        match current_doc_pos {
                            0 => {
                                // Chapter index
                                Some(ConfigItem::new(
                                    Some(part.id.clone()),
                                    Some(chapter.id.clone()),
                                    Some(part_idx),
                                    Some(chapter_idx),
                                    chapter.index.clone(),
                                    Some(chapter.files.clone()),
                                ))
                            }
                            doc_pos => Some(ConfigItem::new(
                                Some(part.id.clone()),
                                Some(chapter.id.clone()),
                                Some(part_idx),
                                Some(chapter_idx),
                                chapter.documents[doc_pos - 1].clone(),
                                Some(chapter.files.clone()),
                            )),
                        }
                    }
                }
            }
            _ => None,
        }
    }
}

/// Specifies the document format. Currently, only Markdown and Notebooks (ipynb) are supported.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Format {
    Markdown,
    Notebook,
}

/// A part is the highest level of content division. Each project has a series of parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part<C> {
    /// Part id (folder name)
    pub id: String,
    /// Index document
    pub index: DocumentSpec<C>,
    /// Chapters (in order)
    pub chapters: Vec<Chapter<C>>,
}

/// Parts contain chapters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter<C> {
    /// Chapter id (folder name)
    pub id: String,
    /// Index document
    pub index: DocumentSpec<C>,
    /// Individual documents
    pub documents: Vec<DocumentSpec<C>>,
    /// Other files
    pub files: Vec<PathBuf>,
}

/// Chapters contain documents. Their configuration container is called DocumentSpec. It is a generic
/// type over the inner "document". This is useful for using the configuration as a datastructure
/// for content throughout the build process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSpec<C> {
    /// Document id (filename excluding extension)
    pub id: String,
    /// Document source format
    pub format: Format,
    /// Location
    pub path: PathBuf,
    /// Content. It is wrapped in Arc to minimise unnecessary memory copying.
    pub content: Arc<C>,
}

/// The top-level configuration of a project's content.TTT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config<C> {
    pub project_path: PathBuf,
    pub(crate) index: DocumentSpec<C>,
    pub(crate) content: Vec<Part<C>>,
}

/// Refers to a configuration.yml file in the project that specifies a variety
/// of options for the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub url_prefix: String,
    #[serde(default)]
    pub build: BuildConfigSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfigSet {
    pub dev: BuildConfig,
    pub release: BuildConfig,
}

impl BuildConfigSet {
    pub fn get_config(&self, mode: &str) -> anyhow::Result<BuildConfig> {
        match mode {
            "dev" => Ok(self.dev.clone()),
            "release" => Ok(self.release.clone()),
            _ => Err(anyhow!("Invalid build mode")),
        }
    }
}

impl Default for BuildConfigSet {
    fn default() -> Self {
        BuildConfigSet {
            dev: BuildConfig {
                katex_output: false,
            },
            release: BuildConfig { katex_output: true },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub katex_output: bool,
}

impl<I, O> Transform<Chapter<O>, I, O> for Chapter<I> {
    fn transform<F>(&self, f: &F) -> Chapter<O>
    where
        F: Fn(&DocumentSpec<I>) -> O,
    {
        Chapter {
            id: self.id.clone(),
            index: self.index.transform(f),
            documents: self.documents.iter().map(|d| d.transform(f)).collect(),
            files: self.files.clone(),
        }
    }
}

impl<I> Chapter<I> {
    fn transform_parents_helper<F, O>(&self, part: &Part<I>, f: &F) -> Chapter<O>
    where
        F: Fn(&DocumentSpec<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
    {
        Chapter {
            id: self.id.clone(),
            index: self
                .index
                .transform_parents_helper(Some(part), Some(self), f),
            documents: self
                .documents
                .iter()
                .map(|d| d.transform_parents_helper(Some(part), Some(self), f))
                .collect(),
            files: self.files.clone(),
        }
    }
}

impl<I, O> Transform<Part<O>, I, O> for Part<I> {
    fn transform<F>(&self, f: &F) -> Part<O>
    where
        F: Fn(&DocumentSpec<I>) -> O,
    {
        Part {
            id: self.id.clone(),
            index: self.index.transform(f),
            chapters: self.chapters.iter().map(|c| c.transform(f)).collect(),
        }
    }
}

impl<I, O> TransformParents<Part<O>, I, O> for Part<I> {
    fn transform_parents<F>(&self, f: &F) -> Part<O>
    where
        F: Fn(&DocumentSpec<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
    {
        Part {
            id: self.id.clone(),
            index: self.index.transform_parents_helper(Some(self), None, f),
            chapters: self
                .chapters
                .iter()
                .map(|c| c.transform_parents_helper(self, f))
                .collect(),
        }
    }
}

impl<I, O> Transform<Config<O>, I, O> for Config<I> {
    fn transform<F>(&self, f: &F) -> Config<O>
    where
        F: Fn(&DocumentSpec<I>) -> O,
    {
        Config {
            project_path: self.project_path.clone(),
            index: self.index.transform(f),
            content: self.content.iter().map(|p| p.transform(f)).collect(),
        }
    }
}

impl<I, O> TransformParents<Config<O>, I, O> for Config<I> {
    fn transform_parents<F>(&self, f: &F) -> Config<O>
    where
        F: Fn(&DocumentSpec<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
    {
        Config {
            project_path: self.project_path.clone(),
            index: self.index.transform_parents_helper(None, None, f),
            content: self
                .content
                .iter()
                .map(|p| p.transform_parents(f))
                .collect(),
        }
    }
}

impl<I, O> Transform<DocumentSpec<O>, I, O> for DocumentSpec<I> {
    fn transform<F>(&self, f: &F) -> DocumentSpec<O>
    where
        F: Fn(&DocumentSpec<I>) -> O,
    {
        DocumentSpec {
            id: self.id.clone(),
            format: self.format.clone(),
            path: self.path.clone(),
            content: Arc::new(f(self)),
        }
    }
}

impl<I> DocumentSpec<I> {
    fn transform_parents_helper<F, O>(
        &self,
        part: Option<&Part<I>>,
        chapter: Option<&Chapter<I>>,
        f: &F,
    ) -> DocumentSpec<O>
    where
        F: Fn(&DocumentSpec<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
    {
        DocumentSpec {
            id: self.id.clone(),
            format: self.format.clone(),
            path: self.path.clone(),
            content: Arc::new(f(self, part, chapter)),
        }
    }
}

impl Format {
    /// Get format from path (md/ipynb).
    pub fn from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        match path.as_ref().extension().unwrap().to_str().unwrap() {
            "md" => Ok(Format::Markdown),
            "ipynb" => Ok(Format::Notebook),
            _ => Err(anyhow!("Invalid file extension")),
        }
    }
}

/// Extract a section_id (folder name) from a full path.
pub fn section_id<P: AsRef<Path>>(path: P) -> Option<String> {
    Some(
        path.as_ref()
            .file_name()?
            .to_str()?
            .split('.')
            .into_iter()
            .next()?
            .to_string(),
    )
}

/// Extract a chapter_id (folder name) from a full path.
fn chapter_id<P: AsRef<Path>>(path: P) -> Option<String> {
    Some(path.as_ref().file_name()?.to_str().unwrap().to_string())
}

impl DocumentSpec<()> {
    fn new<P: AsRef<Path>>(section_path: P) -> anyhow::Result<Self> {
        Ok(DocumentSpec {
            id: section_id(section_path.as_ref())
                .ok_or_else(|| anyhow!("Could not get raw file name"))?,
            path: section_path.as_ref().to_path_buf(),
            format: Format::from_path(section_path)?,
            content: Arc::new(()),
        })
    }
}

const EXT: [&str; 2] = ["md", "ipynb"];

fn extension_in(extension: &str) -> bool {
    EXT.iter().any(|e| e == &extension)
}

fn index_helper<P: AsRef<Path>, PC: AsRef<Path>>(
    chapter_dir: &P,
    content_path: &PC,
) -> anyhow::Result<DocumentSpec<()>> {
    let chapter_index_md = chapter_dir.as_ref().join("index.md");
    let chapter_index_ipynb = chapter_dir.as_ref().join("index.ipynb");
    let chapter_index = if chapter_index_md.is_file() {
        chapter_index_md
    } else {
        chapter_index_ipynb
    };

    DocumentSpec::new(chapter_index.strip_prefix(content_path.as_ref())?)
}

impl Chapter<()> {
    fn new<P: AsRef<Path>, PC: AsRef<Path>>(
        chapter_dir: P,
        content_path: PC,
    ) -> anyhow::Result<Self> {
        let section_dir = chapter_dir.as_ref();

        let paths = get_sorted_paths(section_dir)?
            .into_iter()
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .filter(|e| extension_in(e.to_str().unwrap()))
                    .is_some()
            })
            .filter(|entry| !entry.file_name().to_str().unwrap().contains("index"))
            .filter(|entry| entry.metadata().map(|meta| meta.is_file()).is_ok());

        let file_paths = get_sorted_paths(section_dir)?
            .into_iter()
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .filter(|e| extension_in(e.to_str().unwrap()))
                    .is_none()
            })
            .filter(|entry| !entry.file_name().to_str().unwrap().contains("index"))
            .filter(|entry| entry.metadata().map(|meta| meta.is_file()).is_ok())
            .map(|entry| entry.path())
            .collect();

        let documents: Vec<DocumentSpec<()>> = paths
            .map(|entry| DocumentSpec::new(entry.path().strip_prefix(content_path.as_ref())?))
            .collect::<anyhow::Result<Vec<DocumentSpec<()>>>>()?;

        let index_doc = index_helper(&chapter_dir, &content_path);

        Ok(Chapter {
            id: chapter_id(chapter_dir).ok_or_else(|| anyhow!("Can't get chapter id"))?,
            index: index_doc?,
            documents,
            files: file_paths,
        })
    }
}

impl Part<()> {
    fn new<P: AsRef<Path>, PC: AsRef<Path>>(dir: P, content_path: PC) -> anyhow::Result<Self> {
        let part_folder = chapter_id(&dir).ok_or_else(|| anyhow!("Can't get part id"))?;
        // let part_dir = dir.as_ref().join(&part_folder);

        let chapters = get_sorted_paths(&dir)?
            .into_iter()
            .filter(|entry| entry.metadata().map(|meta| meta.is_dir()).unwrap())
            .map(|entry| Chapter::new(entry.path(), content_path.as_ref()))
            .collect::<anyhow::Result<Vec<Chapter<()>>>>()?;

        Ok(Part {
            id: part_folder,
            index: index_helper(&dir, &content_path)?,
            chapters,
        })
    }
}

fn get_sorted_paths<P: AsRef<Path>>(path: P) -> io::Result<Vec<DirEntry>> {
    let mut paths: Vec<_> = fs::read_dir(&path)?.filter_map(|r| r.ok()).collect();
    paths.sort_by_key(|p| p.path());
    Ok(paths)
}

impl Config<()> {
    /// Construct configuration from a directory (generally the project directory). The function
    /// finds and verifies the structure of the project.
    pub fn generate_from_directory<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content_path = path.as_ref().join("content");

        let parts = get_sorted_paths(&content_path)?
            .into_iter()
            .filter_map(|entry| {
                let m = fs::metadata(entry.path());
                m.map(|m| m.is_dir().then_some(entry)).ok()?
            })
            .map(|entry| {
                let file_path = entry.path();
                Part::new(file_path, content_path.as_path())
            })
            .collect::<anyhow::Result<Vec<Part<()>>>>()?;

        let index_doc = index_helper(&content_path, &content_path)?;

        Ok(Config {
            project_path: path.as_ref().to_path_buf(),
            index: index_doc,
            content: parts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::zip;

    #[test]
    fn gen_config_from_dir() {
        let cfg = Config::generate_from_directory("resources/test").expect("Could not read config");
        assert_eq!(cfg.content.len(), 1); // 1 part
        assert_eq!(cfg.content[0].chapters.len(), 4); // 4 chapters in part 1
        assert_eq!(cfg.content[0].chapters[0].id, "01_getting_started");
        assert_eq!(cfg.content[0].chapters[1].id, "02_project_organisation");
        assert_eq!(cfg.content[0].chapters[2].id, "03_shortcodes");
        assert_eq!(cfg.content[0].chapters[3].id, "04_exercise_tools");
    }

    #[test]
    fn test_iteration_collect() {
        let doc = DocumentSpec {
            id: "doc".to_string(),
            format: Format::Markdown,
            path: Default::default(),
            content: Arc::new(()),
        };

        let cfg = Config {
            project_path: Default::default(),
            index: doc.clone(),
            content: vec![
                Part {
                    id: "part1".to_string(),
                    index: doc.clone(),
                    chapters: vec![
                        Chapter {
                            id: "chapter1".to_string(),
                            index: doc.clone(),
                            documents: vec![doc.clone(), doc.clone()],
                            files: vec![PathBuf::new()],
                        },
                        Chapter {
                            id: "chapter2".to_string(),
                            index: doc.clone(),
                            documents: vec![doc.clone(), doc.clone()],
                            files: vec![PathBuf::new()],
                        },
                    ],
                },
                Part {
                    id: "part2".to_string(),
                    index: doc,
                    chapters: vec![],
                },
            ],
        };

        let cfg_mapped: Config<()> = cfg.clone().into_iter().collect();

        for (p1, p2) in zip(cfg.content, cfg_mapped.content) {
            assert_eq!(p1.id, p2.id);
        }
    }
}
