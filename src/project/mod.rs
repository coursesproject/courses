use std::ffi::OsStr;
use std::fs::DirEntry;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};

use cdoc::config::InputFormat;
use cdoc::document::Document;
use cdoc::renderers::RenderResult;
pub use iterator::*;
pub use transform::*;

pub mod config;

mod iterator;
mod transform;

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentDescriptor<C> {
    pub(crate) id: String,
    pub(crate) format: InputFormat,
    pub(crate) path: PathBuf,
    pub(crate) content: Arc<C>,
}

impl<C> Clone for DocumentDescriptor<C> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            format: self.format.clone(),
            path: self.path.clone(),
            content: self.content.clone(),
        }
    }
}

impl DocumentDescriptor<()> {
    fn new<P: AsRef<Path>>(section_path: P) -> anyhow::Result<Self> {
        Ok(DocumentDescriptor {
            id: section_id(section_path.as_ref())
                .ok_or_else(|| anyhow!("Could not get raw file name"))?,
            format: InputFormat::from_extension(
                section_path.as_ref().extension().unwrap().to_str().unwrap(),
            )?,
            path: section_path.as_ref().to_path_buf(),
            content: Arc::new(()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentItem<C> {
    Document {
        doc: DocumentDescriptor<C>,
    },
    Section {
        id: String,
        doc: DocumentDescriptor<C>,
        children: Vec<ContentItem<C>>,
    },
}

impl<C> DocumentDescriptor<C> {
    pub fn as_ref(&self) -> DocumentDescriptor<&C> {
        DocumentDescriptor {
            id: self.id.clone(),
            format: self.format,
            path: self.path.clone(),
            content: Arc::new(self.content.as_ref()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContentItemDescriptor<C> {
    pub path: Vec<String>,
    pub path_idx: Vec<usize>,
    pub doc: DocumentDescriptor<C>,
}

impl<C> ContentItemDescriptor<C> {
    /// Perform operation on the inner document, then return the result wrapped in a ConfigItem.
    pub fn map<O, F>(self, f: F) -> anyhow::Result<ContentItemDescriptor<O>>
    where
        F: Fn(&C) -> anyhow::Result<O>,
    {
        Ok(ContentItemDescriptor {
            path: self.path,
            path_idx: self.path_idx,
            doc: DocumentDescriptor {
                id: self.doc.id,
                format: self.doc.format,
                path: self.doc.path,
                content: Arc::new(f(self.doc.content.as_ref())?),
            },
        })
    }

    /// Perform operation on the whole DocumentSpec.
    pub fn map_doc<O, F, E>(self, f: F) -> Result<ContentItemDescriptor<O>, E>
    where
        F: Fn(DocumentDescriptor<C>) -> Result<O, E>,
    {
        Ok(ContentItemDescriptor {
            path: self.path,
            path_idx: self.path_idx,
            doc: DocumentDescriptor {
                id: self.doc.id.clone(),
                format: self.doc.format.clone(),
                path: self.doc.path.clone(),
                content: Arc::new(f(self.doc)?),
            },
        })
    }
}

pub fn from_vec<C: Clone>(vec: &Vec<ContentItemDescriptor<C>>) -> ContentItem<C> {
    let mut current_idx = 0;
    let mut current_lvl = 0;
    let res = from_vec_helper(&vec, &mut current_idx, &mut current_lvl, 1)
        .unwrap()
        .remove(0);
    res.clone()
}

fn from_vec_helper<C>(
    vec: &Vec<ContentItemDescriptor<C>>,
    mut current_idx: &mut usize,
    mut current_lvl: &mut usize,
    current_path_len: usize,
) -> Option<Vec<ContentItem<C>>> {
    // let section_item = iter.next().unwrap();

    let mut children = Vec::new();
    while *current_idx < vec.len() {
        let next = &vec[*current_idx];

        if next.path.last().unwrap() == "index" {
            if current_path_len < next.path.len() {
                *current_lvl += 1;
                *current_idx += 1;
            } else if current_path_len >= next.path.len() {
                *current_lvl -= 1;
                return Some(children);
            }
            println!("{:?}, {}, {}", next.path, current_idx, current_lvl);

            // if next_level == current_lvl {
            //     println!("same");
            //     return None;
            // } else {

            let inner =
                from_vec_helper(vec, current_idx, current_lvl, next.path.len()).unwrap_or_default();
            println!("new section");
            children.push(ContentItem::Section {
                id: next.path.get(next.path.len() - 2).unwrap().clone(),
                doc: next.doc.clone(),
                children: inner,
            });
        } else {
            children.push(ContentItem::Document {
                doc: next.doc.clone(),
            });
            *current_idx += 1;
        }
    }

    Some(children)
}

impl<C> Hash for ContentItemDescriptor<C>
where
    C: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state)
    }
}

impl<C> ContentItem<C> {
    pub fn doc_at_idx(&self, path_idx: &[usize]) -> anyhow::Result<&DocumentDescriptor<C>> {
        match self {
            ContentItem::Document { doc } => Ok(doc),
            ContentItem::Section { children, .. } => {
                let child = children
                    .get(*path_idx.get(0).ok_or(anyhow!("invalid path"))?)
                    .ok_or(anyhow!("invalid path"))?;
                if path_idx.len() > 1 {
                    Ok(child.doc_at_idx(&path_idx[1..])?)
                } else {
                    Ok(child.get_doc())
                }
            }
        }
    }

    pub fn get_doc(&self) -> &DocumentDescriptor<C> {
        match self {
            ContentItem::Document { doc } => doc,
            ContentItem::Section { doc: index, .. } => index,
        }
    }

    fn get_path_idx_inner(&self, path: &[String], path_idx: &mut Vec<usize>) -> Option<Vec<usize>> {
        match self {
            ContentItem::Document { doc } => {
                if &doc.id == path.last()? {
                    Some(path_idx.clone())
                } else {
                    None
                }
            }
            ContentItem::Section {
                id,
                doc: index,
                children,
            } => {
                path_idx.push(0);
                children
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, child)| {
                        path_idx.last_mut().map(|mut idx| *idx = i);
                        self.get_path_idx_inner(path, path_idx)
                    })
                    .next()
            }
        }
    }

    pub fn get_path_idx(&self, path: &[String]) -> Option<Vec<usize>> {
        let mut path_idx = vec![0];
        self.get_path_idx_inner(path, &mut path_idx)
    }

    fn add_doc(
        doc: DocumentDescriptor<C>,
        current_path: &mut Vec<String>,
        current_path_idx: &mut Vec<usize>,
    ) -> ContentItemDescriptor<C> {
        if let Some(mut last) = current_path.last_mut() {
            *last = doc.id.clone();
        }

        let content = ContentItemDescriptor {
            path: current_path.clone(),
            path_idx: current_path_idx.clone(),
            doc: doc.clone(),
        };

        if let Some(mut last) = current_path_idx.last_mut() {
            *last += 1;
        }

        content
    }
    fn add_to_vector(
        self,
        mut current_path: &mut Vec<String>,
        mut current_path_idx: &mut Vec<usize>,
        vec: &mut Vec<ContentItemDescriptor<C>>,
    ) {
        // println!("{:?}, {:?}", current_path, current_path_idx);
        match self {
            ContentItem::Document { doc } => vec.push(ContentItem::add_doc(
                doc.clone(),
                current_path,
                current_path_idx,
            )),
            ContentItem::Section {
                id,
                doc: index,
                children,
            } => {
                if let Some(mut last) = current_path.last_mut() {
                    *last = id.clone();
                }

                let path_idx_index = current_path_idx.len() - 1;
                // println!("{}", path_idx_index);

                current_path.push("index".to_string());
                current_path_idx.push(0);

                vec.push(ContentItem::add_doc(
                    index,
                    &mut current_path,
                    &mut current_path_idx,
                ));

                for child in children {
                    child.add_to_vector(&mut current_path, &mut current_path_idx, vec);
                }

                let mut idx = current_path_idx.get_mut(path_idx_index).unwrap();
                *idx += 1;

                current_path.pop();
                current_path_idx.pop();
            }
        }
    }
    pub fn to_vector(self) -> Vec<ContentItemDescriptor<C>> {
        let mut output = Vec::new();
        let mut c_path = vec!["".to_string()];
        let mut c_path_idx = vec![0];
        self.add_to_vector(&mut c_path, &mut c_path_idx, &mut output);
        output
    }
}

pub fn configure_project(path: PathBuf) -> anyhow::Result<ContentItem<()>> {
    Ok(ContentItem::Section {
        id: "root".to_string(),
        doc: index_helper2(&path, &path)?,
        children: content_from_dir(path.clone(), path)?,
    })
}

pub fn content_from_dir(
    path: PathBuf,
    content_path: PathBuf,
) -> anyhow::Result<Vec<ContentItem<()>>> {
    let content: anyhow::Result<Vec<ContentItem<()>>> = get_sorted_paths(path)?
        .into_iter()
        .filter(|d| {
            let p = d.path();
            p.is_dir()
                || match p.extension() {
                    None => false,
                    Some(e) => extension_in(e.to_str().unwrap()),
                }
        })
        .filter(|entry| !entry.file_name().to_str().unwrap().contains("index"))
        .filter_map(|d| {
            let p = d.path();
            if p.is_dir() {
                match index_helper2(&p, &content_path) {
                    Ok(ix) => Some(create_section(content_path.clone(), &p, ix)),
                    Err(_) => None,
                }
            } else {
                match DocumentDescriptor::new(&p) {
                    Ok(doc) => Some(Ok(ContentItem::Document { doc })),
                    Err(e) => Some(Err(e)),
                }
            }
        })
        .collect();

    content
}

fn create_section(
    content_path: PathBuf,
    p: &PathBuf,
    ix: DocumentDescriptor<()>,
) -> anyhow::Result<ContentItem<()>> {
    Ok(ContentItem::Section {
        id: chapter_id(&p).ok_or(anyhow!("no id"))?,
        doc: ix,
        children: content_from_dir(p.clone(), content_path.clone())?,
    })
}

/// The top-level configuration of a project's content.TTT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project<C> {
    pub project_path: PathBuf,
    pub(crate) index: ProjectItem<C>,
    pub(crate) content: Vec<Part<C>>,
}

/// A part is the highest level of content division. Each project has a series of parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part<C> {
    /// Part id (folder name)
    pub id: String,
    /// Index document
    pub index: ProjectItem<C>,
    /// Chapters (in order)
    pub chapters: Vec<Chapter<C>>,
}

/// Parts contain chapters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter<C> {
    /// Chapter id (folder name)
    pub id: String,
    /// Index document
    pub index: ProjectItem<C>,
    /// Individual documents
    pub documents: Vec<ProjectItem<C>>,
    /// Other files
    pub files: Vec<PathBuf>,
}

/// Chapters contain documents. Their configuration container is called DocumentSpec. It is a generic
/// type over the inner "document". This is useful for using the configuration as a datastructure
/// for content throughout the build process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectItem<C> {
    /// Document id (filename excluding extension)
    pub id: String,
    /// Document source format
    pub format: InputFormat,
    /// Location
    pub path: PathBuf,
    /// Content. It is wrapped in Arc to minimise unnecessary memory copying.
    pub content: Arc<C>,
}

impl<C> ProjectItem<C> {
    pub fn as_ref(&self) -> ProjectItem<&C> {
        ProjectItem {
            id: self.id.clone(),
            format: self.format,
            path: self.path.clone(),
            content: Arc::new(self.content.as_ref()),
        }
    }
}
/// Iterates a Config.
pub struct ProjectIterator<D> {
    part_pos: usize,
    chapter_pos: usize,
    doc_pos: usize,
    config: Project<D>,
}

pub type ContentResult<'a> = ContentItem<&'a Option<Document<RenderResult>>>;
pub type ContentResultS = ContentItem<Option<Document<RenderResult>>>;
pub type ProjectItemVec = Vec<ContentItemDescriptor<Option<Document<RenderResult>>>>;

/// Contains necessary information for reconstructing a Config from an iterator.
#[derive(Clone, Debug)]
pub struct ItemDescriptor<D> {
    pub part_id: Option<String>,
    pub chapter_id: Option<String>,
    pub part_idx: Option<usize>,
    pub chapter_idx: Option<usize>,
    pub doc_idx: Option<usize>,
    pub doc: ProjectItem<D>,
    pub files: Option<Vec<PathBuf>>, // Temporary solution for carrying file info
}

impl<C> Project<C> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<C> Part<C> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        1 + self.chapters.iter().map(|c| c.len()).sum::<usize>()
    }
}

impl<C> Chapter<C> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        1 + self.documents.iter().map(|_| 1).sum::<usize>()
    }
}

// impl<C> ProjectItem<C> {
//     pub fn len(&self) -> usize {
//         1
//     }
// }

impl<C> Project<C> {
    pub fn len(&self) -> usize {
        1 + self.content.iter().map(|p| p.len()).sum::<usize>()
    }
}

impl Project<()> {
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

        Ok(Project {
            project_path: path.as_ref().to_path_buf(),
            index: index_doc,
            content: parts,
        })
    }
}

impl Part<()> {
    fn new<P: AsRef<Path>, PC: AsRef<Path>>(path: P, content_path: PC) -> anyhow::Result<Self> {
        let part_folder = chapter_id(&path).ok_or_else(|| anyhow!("Can't get part id"))?;
        // let part_dir = dir.as_ref().join(&part_folder);

        let chapters = get_sorted_paths(&path)?
            .into_iter()
            .filter(|entry| entry.metadata().map(|meta| meta.is_dir()).unwrap())
            .map(|entry| Chapter::new(entry.path(), content_path.as_ref()))
            .collect::<anyhow::Result<Vec<Chapter<()>>>>()?;

        Ok(Part {
            id: part_folder,
            index: index_helper(&path, &content_path)?,
            chapters,
        })
    }
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

        let documents: Vec<ProjectItem<()>> = paths
            .map(|entry| ProjectItem::new(entry.path().strip_prefix(content_path.as_ref())?))
            .collect::<anyhow::Result<Vec<ProjectItem<()>>>>()?;

        let index_doc = index_helper(&chapter_dir, &content_path);

        Ok(Chapter {
            id: chapter_id(chapter_dir).ok_or_else(|| anyhow!("Can't get chapter id"))?,
            index: index_doc?,
            documents,
            files: file_paths,
        })
    }
}

impl<I> ProjectItem<I> {
    fn transform_parents_helper<F, O>(
        &self,
        part: Option<&Part<I>>,
        chapter: Option<&Chapter<I>>,
        f: &F,
    ) -> ProjectItem<O>
    where
        F: Fn(&ProjectItem<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
    {
        ProjectItem {
            id: self.id.clone(),
            format: self.format,
            path: self.path.clone(),
            content: Arc::new(f(self, part, chapter)),
        }
    }
}

impl<C> ProjectItem<C> {
    pub fn map<O, F>(self, f: F) -> ProjectItem<O>
    where
        F: Fn(&C) -> O,
    {
        ProjectItem {
            id: self.id,
            format: self.format,
            path: self.path,
            content: Arc::new(f(self.content.deref())),
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
            .next()?
            .to_string(),
    )
}

/// Extract a chapter_id (folder name) from a full path.
fn chapter_id<P: AsRef<Path>>(path: P) -> Option<String> {
    Some(path.as_ref().file_name()?.to_str().unwrap().to_string())
}

impl ProjectItem<()> {
    fn new<P: AsRef<Path>>(section_path: P) -> anyhow::Result<Self> {
        Ok(ProjectItem {
            id: section_id(section_path.as_ref())
                .ok_or_else(|| anyhow!("Could not get raw file name"))?,
            path: section_path.as_ref().to_path_buf(),
            format: InputFormat::from_extension(
                section_path.as_ref().extension().unwrap().to_str().unwrap(),
            )?,
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
) -> anyhow::Result<ProjectItem<()>> {
    let chapter_index_md = chapter_dir.as_ref().join("index.md");
    let chapter_index_ipynb = chapter_dir.as_ref().join("index.ipynb");
    let chapter_index = if chapter_index_md.is_file() {
        Ok(chapter_index_md)
    } else if chapter_index_ipynb.is_file() {
        Ok(chapter_index_ipynb)
    } else {
        Err(anyhow!(
            "index.md/index.ipynb file not found in path: {}",
            chapter_dir.as_ref().display()
        ))
    }?;

    ProjectItem::new(chapter_index.strip_prefix(content_path.as_ref())?)
}

fn index_helper2<P: AsRef<Path>, PC: AsRef<Path>>(
    chapter_dir: &P,
    content_path: &PC,
) -> anyhow::Result<DocumentDescriptor<()>> {
    let chapter_index_md = chapter_dir.as_ref().join("index.md");
    let chapter_index_ipynb = chapter_dir.as_ref().join("index.ipynb");
    let chapter_index = if chapter_index_md.is_file() {
        Ok(chapter_index_md)
    } else if chapter_index_ipynb.is_file() {
        Ok(chapter_index_ipynb)
    } else {
        Err(anyhow!(
            "index.md/index.ipynb file not found in path: {}",
            chapter_dir.as_ref().display()
        ))
    }?;

    DocumentDescriptor::new(chapter_index.strip_prefix(content_path.as_ref())?)
}

fn get_sorted_paths<P: AsRef<Path>>(path: P) -> io::Result<Vec<DirEntry>> {
    // println!("{}", path.as_ref().display());
    let mut paths: Vec<_> = fs::read_dir(&path)?.filter_map(|r| r.ok()).collect();
    paths.sort_by_key(|p| p.path());
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::iter::zip;

    use super::*;

    #[test]
    fn gen_new_config() {
        let cfg = configure_project(Path::new("docs/content").to_path_buf())
            .expect("Could not read config");
        // assert_eq!(cfg.len(), 4);
        println!("{:?}", cfg);
    }

    #[test]
    fn config_to_vector() {
        let cfg = configure_project(
            Path::new("/Users/anton/git/iaml/exercises/coursematerial/content").to_path_buf(),
        )
        .expect("Could not read config");
        let v = cfg.to_vector();
        println!("{:#?}", v);
    }

    #[test]
    fn gen_config_from_dir() {
        let cfg =
            Project::generate_from_directory("resources/test").expect("Could not read config");
        assert_eq!(cfg.content.len(), 1); // 1 part
        assert_eq!(cfg.content[0].chapters.len(), 4); // 4 chapters in part 1
        assert_eq!(cfg.content[0].chapters[0].id, "01_getting_started");
        assert_eq!(cfg.content[0].chapters[1].id, "02_project_organisation");
        assert_eq!(cfg.content[0].chapters[2].id, "03_shortcodes");
        assert_eq!(cfg.content[0].chapters[3].id, "04_exercise_tools");
    }

    #[test]
    fn test_iteration_collect() {
        let doc = ProjectItem {
            id: "doc".to_string(),
            format: InputFormat::Markdown,
            path: Default::default(),
            content: Arc::new(()),
        };

        let cfg = Project {
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

        let cfg_mapped: Project<()> = cfg.clone().into_iter().collect();

        for (p1, p2) in zip(cfg.content, cfg_mapped.content) {
            assert_eq!(p1.id, p2.id);
        }
    }
}
