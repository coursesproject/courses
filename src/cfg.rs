use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub trait Transform<T, I, O> {
    fn transform<F>(&self, f: &F) -> T
    where
        F: Fn(&Document<I>) -> O;
}

pub trait TransformParents<T, I, O> {
    fn transform_parents<F>(&self, f: &F) -> T
    where
        F: Fn(&Document<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Format {
    Markdown,
    Notebook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part<C> {
    pub(crate) id: String,
    pub(crate) index: Document<C>,
    pub(crate) chapters: Vec<Chapter<C>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter<C> {
    pub(crate) id: String,
    pub(crate) index: Document<C>,
    pub(crate) documents: Vec<Document<C>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document<C> {
    pub(crate) id: String,
    pub(crate) format: Format,
    pub(crate) path: PathBuf,
    pub(crate) content: C,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig<C> {
    pub(crate) project_path: PathBuf,
    pub(crate) index: Document<C>,
    pub(crate) content: Vec<Part<C>>,
}

impl<I, O> Transform<Chapter<O>, I, O> for Chapter<I> {
    fn transform<F>(&self, f: &F) -> Chapter<O>
    where
        F: Fn(&Document<I>) -> O,
    {
        Chapter {
            id: self.id.clone(),
            index: self.index.transform(f),
            documents: self.documents.iter().map(|d| d.transform(f)).collect(),
        }
    }
}

impl<I> Chapter<I> {
    fn transform_parents_helper<F, O>(&self, part: &Part<I>, f: &F) -> Chapter<O>
    where
        F: Fn(&Document<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
    {
        Chapter {
            id: self.id.clone(),
            index: self
                .index
                .transform_parents_helper(Some(part), Some(&self), f),
            documents: self
                .documents
                .iter()
                .map(|d| d.transform_parents_helper(Some(part), Some(&self), f))
                .collect(),
        }
    }
}

impl<I, O> Transform<Part<O>, I, O> for Part<I> {
    fn transform<F>(&self, f: &F) -> Part<O>
    where
        F: Fn(&Document<I>) -> O,
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
        F: Fn(&Document<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
    {
        Part {
            id: self.id.clone(),
            index: self.index.transform_parents_helper(Some(self), None, f),
            chapters: self
                .chapters
                .iter()
                .map(|c| c.transform_parents_helper(&self, f))
                .collect(),
        }
    }
}

impl<I, O> Transform<BuildConfig<O>, I, O> for BuildConfig<I> {
    fn transform<F>(&self, f: &F) -> BuildConfig<O>
    where
        F: Fn(&Document<I>) -> O,
    {
        BuildConfig {
            project_path: self.project_path.clone(),
            index: self.index.transform(f),
            content: self.content.iter().map(|p| p.transform(f)).collect(),
        }
    }
}

impl<I, O> TransformParents<BuildConfig<O>, I, O> for BuildConfig<I> {
    fn transform_parents<F>(&self, f: &F) -> BuildConfig<O>
    where
        F: Fn(&Document<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
    {
        BuildConfig {
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

impl<I, O> Transform<Document<O>, I, O> for Document<I> {
    fn transform<F>(&self, f: &F) -> Document<O>
    where
        F: Fn(&Document<I>) -> O,
    {
        Document {
            id: self.id.clone(),
            format: self.format.clone(),
            path: self.path.clone(),
            content: f(self),
        }
    }
}

impl<I> Document<I> {
    fn transform_parents_helper<F, O>(
        &self,
        part: Option<&Part<I>>,
        chapter: Option<&Chapter<I>>,
        f: &F,
    ) -> Document<O>
    where
        F: Fn(&Document<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
    {
        Document {
            id: self.id.clone(),
            format: self.format.clone(),
            path: self.path.clone(),
            content: f(self, part, chapter),
        }
    }
}

impl Format {
    pub fn from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        match path.as_ref().extension().unwrap().to_str().unwrap() {
            "md" => Ok(Format::Markdown),
            "ipynb" => Ok(Format::Notebook),
            _ => Err(anyhow!("Invalid file extension")),
        }
    }
}

pub fn section_id<P: AsRef<Path>>(path: P) -> Option<String> {
    Some(
        path.as_ref()
            .file_name()?
            .to_str()?
            .split(".")
            .into_iter()
            .next()?
            .to_string(),
    )
}

fn chapter_id<P: AsRef<Path>>(path: P) -> Option<String> {
    Some(path.as_ref().file_name()?.to_str().unwrap().to_string())
}

impl Document<()> {
    fn new<P: AsRef<Path>>(section_path: P) -> anyhow::Result<Self> {
        Ok(Document {
            id: section_id(section_path.as_ref()).ok_or(anyhow!("Could not get raw file name"))?,
            path: section_path.as_ref().to_path_buf(),
            format: Format::from_path(section_path)?,
            content: (),
        })
    }
}

const EXT: [&str; 2] = ["md", "ipynb"];

fn extension_in(extension: &str) -> bool {
    EXT.iter().any(|e| e == &extension)
}

fn index_helper<P: AsRef<Path>>(chapter_dir: &P) -> anyhow::Result<Document<()>> {
    let chapter_index_md = chapter_dir.as_ref().join("index.md");
    let chapter_index_ipynb = chapter_dir.as_ref().join("index.ipynb");
    let chapter_index = if chapter_index_md.is_file() {
        chapter_index_md
    } else {
        chapter_index_ipynb
    };

    Document::new(chapter_index)
}

impl Chapter<()> {
    fn new<P: AsRef<Path>>(chapter_dir: P) -> anyhow::Result<Self> {
        let section_dir = chapter_dir.as_ref();
        println!("d {:?}", section_dir);
        let dirs = fs::read_dir(section_dir)?;
        let paths = dirs
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .filter(|e| extension_in(e.to_str().unwrap()))
                    .is_some()
            })
            .filter(|entry| !entry.file_name().to_str().unwrap().contains("index"))
            .filter(|entry| entry.metadata().map(|meta| meta.is_file()).is_ok());

        let documents: Vec<Document<()>> = paths
            .map(|entry| Document::new(entry.path()))
            .collect::<anyhow::Result<Vec<Document<()>>>>()?;

        let index_doc = index_helper(&chapter_dir);

        Ok(Chapter {
            id: chapter_id(chapter_dir).ok_or(anyhow!("Can't get chapter id"))?,
            index: index_doc?,
            documents,
        })
    }
}

impl Part<()> {
    fn new<P: AsRef<Path>>(dir: P) -> anyhow::Result<Self> {
        let part_folder = chapter_id(&dir).ok_or(anyhow!("Can't get part id"))?;
        // let part_dir = dir.as_ref().join(&part_folder);

        let chapters = match fs::read_dir(&dir) {
            Ok(dir) => {
                let paths = dir
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.metadata().map(|meta| meta.is_dir()).unwrap());

                paths
                    .map(|entry| Chapter::new(entry.path()))
                    .collect::<anyhow::Result<Vec<Chapter<()>>>>()?
            }
            Err(_) => Vec::new(),
        };

        Ok(Part {
            id: part_folder,
            index: index_helper(&dir)?,
            chapters,
        })
    }
}

impl BuildConfig<()> {
    pub fn generate_from_directory<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content_path = path.as_ref().join("content");
        let parts = fs::read_dir(&content_path)?
            .filter_map(|entry| {
                entry
                    .map(|entry| {
                        let m = fs::metadata(entry.path());
                        m.map(|m| m.is_dir().then_some(entry)).ok()?
                    })
                    .ok()?
            })
            .map(|entry| {
                let file_path = entry.path();
                Part::new(file_path)
            })
            .collect::<anyhow::Result<Vec<Part<()>>>>()?;

        let index_doc = index_helper(&content_path)?;

        Ok(BuildConfig {
            project_path: path.as_ref().to_path_buf(),
            index: index_doc,
            content: parts,
        })
    }
}
