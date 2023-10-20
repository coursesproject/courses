use std::fmt::Debug;
use std::fs::DirEntry;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use cdoc::config::InputFormat;
use cdoc::renderers::RenderResult;
use cdoc_base::document::Document;

pub mod caching;
pub mod config;

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
            format: self.format,
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

    fn new_with_id<P: AsRef<Path>>(section_path: P, id: String) -> anyhow::Result<Self> {
        Ok(DocumentDescriptor {
            id,
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
        #[serde(flatten)]
        doc: DocumentDescriptor<C>,
    },
    Section {
        section_id: String,
        section_path: String,
        #[serde(flatten)]
        doc: DocumentDescriptor<C>,
        children: Vec<ContentItem<C>>,
    },
}

// impl<C> ContentItem<C> {
//     pub fn get_path(&self, path: &[&str]) -> Option<ContentItem<C>> {
//         match self {
//             ContentItem::Document { .. } => Some(self.clone()),
//             ContentItem::Section { children } => {}
//         }
//     }
// }

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
    pub is_section: bool,
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
            is_section: self.is_section,
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

    pub fn map_ref<O, F>(&self, f: F) -> anyhow::Result<ContentItemDescriptor<O>>
    where
        F: Fn(&C) -> anyhow::Result<O>,
    {
        Ok(ContentItemDescriptor {
            is_section: self.is_section,
            path: self.path.clone(),
            path_idx: self.path_idx.clone(),
            doc: DocumentDescriptor {
                id: self.doc.id.clone(),
                format: self.doc.format,
                path: self.doc.path.clone(),
                content: Arc::new(f(self.doc.content.as_ref())?),
            },
        })
    }

    /// Perform operation on the whole DocumentSpec.
    pub fn map_doc<O, F, E>(self, mut f: F) -> Result<ContentItemDescriptor<O>, E>
    where
        F: FnMut(DocumentDescriptor<C>) -> Result<O, E>,
    {
        Ok(ContentItemDescriptor {
            is_section: self.is_section,
            path: self.path,
            path_idx: self.path_idx,
            doc: DocumentDescriptor {
                id: self.doc.id.clone(),
                format: self.doc.format,
                path: self.doc.path.clone(),
                content: Arc::new(f(self.doc)?),
            },
        })
    }
}

pub fn from_vec<C: Clone>(vec: &Vec<ContentItemDescriptor<C>>) -> ContentItem<C> {
    let mut current_idx = 0;
    let mut current_lvl = 0;
    from_vec_helper(vec, &mut current_idx, &mut current_lvl, 1).remove(0)
}

fn from_vec_helper<C>(
    vec: &Vec<ContentItemDescriptor<C>>,
    current_idx: &mut usize,
    current_lvl: &mut usize,
    current_path_len: usize,
) -> Vec<ContentItem<C>> {
    // let section_item = iter.next().unwrap();

    let mut children = Vec::new();
    while *current_idx < vec.len() {
        let next = &vec[*current_idx];

        if next.is_section {
            if current_path_len < next.path.len() {
                *current_lvl += 1;
                *current_idx += 1;
            } else if current_path_len >= next.path.len() {
                *current_lvl -= 1;
                return children;
            }

            let inner = from_vec_helper(vec, current_idx, current_lvl, next.path.len());
            children.push(ContentItem::Section {
                section_id: next.path.get(next.path.len() - 2).unwrap().clone(),
                section_path: next.path[1..next.path.len() - 1].join("/"),
                doc: next.doc.clone(),
                children: inner,
            });
        } else if current_path_len > next.path.len() {
            *current_lvl -= 1;
            return children;
        } else {
            children.push(ContentItem::Document {
                doc: next.doc.clone(),
            });
            *current_idx += 1;
        }
    }

    children
}

impl<C> Hash for ContentItemDescriptor<C>
where
    C: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state)
    }
}

impl<C: Debug> ContentItem<C> {
    pub fn len(&self) -> usize {
        match self {
            ContentItem::Document { .. } => 1,
            ContentItem::Section { children, .. } => {
                1 + children.iter().map(ContentItem::len).sum::<usize>()
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn doc_at_idx_inner(
        items: &[&ContentItem<C>],
        path_idx: &[usize],
    ) -> anyhow::Result<DocumentDescriptor<C>> {
        // println!("pp {:?}, it {:?}", path_idx, items);
        let cur_item = items
            .get(path_idx[0] - 1)
            .ok_or_else(|| anyhow!(format!("invalid path index {}", path_idx[0])))?;
        // println!("presec {:?}", path_idx);
        match cur_item {
            ContentItem::Document { doc } => Ok(doc.clone()),
            ContentItem::Section { children, doc, .. } => {
                if path_idx[1] == 0 {
                    Ok(doc.clone())
                } else {
                    ContentItem::doc_at_idx_inner(
                        &children.iter().collect::<Vec<_>>()[..],
                        &path_idx[1..],
                    )
                }
            }
        }
    }

    pub fn doc_at_idx(&self, path_idx: &[usize]) -> anyhow::Result<DocumentDescriptor<C>> {
        if let ContentItem::Section { children, doc, .. } = self {
            if path_idx[1] == 0 {
                Ok(doc.clone())
            } else {
                ContentItem::doc_at_idx_inner(
                    &children.iter().collect::<Vec<_>>()[..],
                    &path_idx[1..],
                )
            }
        } else {
            Err(anyhow!("invalid root item"))
        }
    }

    pub fn get_doc(&self) -> &DocumentDescriptor<C> {
        match self {
            ContentItem::Document { doc } => doc,
            ContentItem::Section { doc: index, .. } => index,
        }
    }

    fn get_path_idx_inner(
        items: &[&ContentItem<C>],
        path: &[String],
        mut path_idx: Vec<usize>,
    ) -> Option<Vec<usize>> {
        // println!("path: {:?}, path_idx: {:?}", path, path_idx);
        // println!("{}", items.len());
        for item in items {
            match item {
                ContentItem::Document { doc } => {
                    if doc.id == path[0] {
                        return Some(path_idx.clone());
                    }
                }
                ContentItem::Section {
                    section_id: id,
                    children,
                    ..
                } => {
                    // println!("section -- p: {} id: {}", path[0], id);
                    if &path[0] == id {
                        path_idx.push(0);
                        return if path[1].as_str() == "index" {
                            // println!("is index: {:?}", path_idx);
                            Some(path_idx.clone())
                        } else {
                            // println!("examining children");
                            *path_idx.last_mut().unwrap() = 1;
                            let c = ContentItem::get_path_idx_inner(
                                &children.iter().collect::<Vec<_>>()[..],
                                &path[1..],
                                path_idx,
                            );
                            // println!("c_at_x: {:?}", c);
                            c
                        };
                    }
                }
            }
            *path_idx.last_mut().unwrap() += 1;
        }
        // println!("end at: {:?}", path_idx);
        None
    }

    pub fn get_path_idx(&self, path: &[String]) -> Option<Vec<usize>> {
        let path_idx = vec![0];
        ContentItem::get_path_idx_inner(&[self], path, path_idx)
    }

    fn add_doc(
        doc: DocumentDescriptor<C>,
        current_path: &mut [String],
        current_path_idx: &mut [usize],
        is_section: bool,
    ) -> ContentItemDescriptor<C> {
        if let Some(last) = current_path.last_mut() {
            *last = doc.id.clone();
        }

        let content = ContentItemDescriptor {
            is_section,
            path: current_path.to_vec(),
            path_idx: current_path_idx.to_vec(),
            doc,
        };

        if let Some(last) = current_path_idx.last_mut() {
            *last += 1;
        }

        content
    }
    fn add_to_vector(
        self,
        current_path: &mut Vec<String>,
        current_path_idx: &mut Vec<usize>,
        vec: &mut Vec<ContentItemDescriptor<C>>,
    ) {
        // println!("{:?}, {:?}", current_path, current_path_idx);
        match self {
            ContentItem::Document { doc } => vec.push(ContentItem::add_doc(
                doc,
                current_path,
                current_path_idx,
                false,
            )),
            ContentItem::Section {
                section_id: id,
                doc: index,
                children,
                ..
            } => {
                if let Some(last) = current_path.last_mut() {
                    *last = id;
                }

                let path_idx_index = current_path_idx.len() - 1;
                // println!("{}", path_idx_index);

                current_path.push("index".to_string());
                current_path_idx.push(0);

                vec.push(ContentItem::add_doc(
                    index,
                    current_path,
                    current_path_idx,
                    true,
                ));

                for child in children {
                    child.add_to_vector(current_path, current_path_idx, vec);
                }

                let idx = current_path_idx.get_mut(path_idx_index).unwrap();
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
        section_id: "root".to_string(),
        section_path: "".to_string(),
        doc: index_helper2_root(&path, &path)?,
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
        section_id: chapter_id(p).ok_or(anyhow!("no id"))?,
        section_path: p.to_str().unwrap().to_string(),
        doc: ix,
        children: content_from_dir(p.clone(), content_path)?,
    })
}

pub type ContentResult<'a> = ContentItem<&'a Option<Document<RenderResult>>>;
pub type ContentResultS = ContentItem<Option<Document<RenderResult>>>;
pub type ContentResultX = ContentItem<Option<Document<()>>>;
pub type ProjectItemVec = Vec<ContentItemDescriptor<Option<Document<()>>>>;
pub type ProjectItemContentVec = Vec<ContentItemDescriptor<Option<Document<RenderResult>>>>;
pub type ProjectItemVecErr =
    Vec<anyhow::Result<ContentItemDescriptor<Option<Document<RenderResult>>>>>;

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

const EXT: [&str; 2] = ["md", "ipynb"];

fn extension_in(extension: &str) -> bool {
    EXT.iter().any(|e| e == &extension)
}

fn index_helper2_root<P: AsRef<Path>, PC: AsRef<Path>>(
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

    let id = chapter_id(chapter_dir).ok_or(anyhow!("no id"))?;
    DocumentDescriptor::new_with_id(chapter_index.strip_prefix(content_path.as_ref())?, id)
}

fn get_sorted_paths<P: AsRef<Path>>(path: P) -> io::Result<Vec<DirEntry>> {
    // println!("{}", path.as_ref().display());
    let mut paths: Vec<_> = fs::read_dir(&path)?.filter_map(|r| r.ok()).collect();
    paths.sort_by_key(|p| p.path());
    Ok(paths)
}

#[cfg(test)]
mod tests {

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
        let cfg = configure_project(Path::new("docs/content").to_path_buf())
            .expect("Could not read config");
        let v = cfg.to_vector();
        println!("{:#?}", v);
    }
}
