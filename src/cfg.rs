use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

// impl<'a, D> IntoIterator for &'a Config<D> where D: Clone {
//     type Item = ConfigItem<&'a D>;
//     type IntoIter = ConfigIterator<D>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         ConfigIteratorRef {
//             part_pos: 0,
//             chapter_pos: 0,
//             doc_pos: 0,
//             config: self,
//         }
//     }
// }

impl<D> IntoIterator for Config<D> where D: Clone {
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

// impl<'a, D> IntoIterator for &'a Config<D> where D: Clone {
//     type Item = ConfigItem<&'a D>;
//     type IntoIter = ConfigIterator<&'a D>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         ConfigIterator {
//             part_pos: 0,
//             chapter_pos: 0,
//             doc_pos: 0,
//             config: self,
//         }
//     }
// }

pub struct PartDescriptor {
    id: String,
}

pub struct ConfigIterator<D> {
    part_pos: usize,
    chapter_pos: usize,
    doc_pos: usize,
    config: Config<D>,
}

// pub struct ConfigIteratorRef<'a, D> {
//     part_pos: usize,
//     chapter_pos: usize,
//     doc_pos: usize,
//     config: Config<&'a D>,
// }

pub struct ConfigItem<D> {
    pub part_id: Option<String>,
    pub chapter_id: Option<String>,
    pub part_idx: Option<usize>,
    pub chapter_idx: Option<usize>,
    pub doc: Document<D>,
}

pub struct ConfigItemRef<'a, D> {
    pub part_id: Option<String>,
    pub chapter_id: Option<String>,
    pub part_idx: Option<usize>,
    pub chapter_idx: Option<usize>,
    pub doc: &'a Document<D>,
}

impl<D> ConfigItem<D> {
    fn new(part_id: Option<String>, chapter_id: Option<String>, part_idx: Option<usize>, chapter_idx: Option<usize>, doc: Document<D>) -> Self {
        ConfigItem {
            part_id,
            chapter_id,
            part_idx,
            chapter_idx,
            doc,
        }
    }

    pub fn map<O, F>(self, f: F) -> anyhow::Result<ConfigItem<O>> where F: Fn(&D) -> anyhow::Result<O> {
        let doc = Document {
            id: self.doc.id,
            format: self.doc.format,
            path: self.doc.path,
            content: Arc::new(f(self.doc.content.as_ref())?),
        };
        Ok(ConfigItem::new(self.part_id, self.chapter_id, self.part_idx, self.chapter_idx, doc))
    }

    pub fn map_doc<O, F>(self, f: F) -> anyhow::Result<ConfigItem<O>> where F: Fn(Document<D>) -> anyhow::Result<O> {
        let doc = Document {
            id: self.doc.id.clone(),
            format: self.doc.format.clone(),
            path: self.doc.path.clone(),
            content: Arc::new(f(self.doc)?),
        };
        Ok(ConfigItem::new(self.part_id, self.chapter_id, self.part_idx, self.chapter_idx, doc))
    }
}

// impl<'a, D> ConfigItemRef<'a, D> {
//     fn new(part_id: Option<String>, chapter_id: Option<String>, part_idx: Option<usize>, chapter_idx: Option<usize>, doc: &Document<D>) -> Self {
//         ConfigItemRef {
//             part_id,
//             chapter_id,
//             part_idx,
//             chapter_idx,
//             doc,
//         }
//     }
//
//     pub fn map<O, F>(self, f: F) -> anyhow::Result<ConfigItemRef<'a, O>> where F: Fn(D) -> anyhow::Result<O> {
//         let doc = Document {
//             id: self.doc.id.clone(),
//             format: self.doc.format.clone(),
//             path: self.doc.path.clone(),
//             content: f(&self.doc.content)?,
//         };
//         Ok(ConfigItemRef::new(self.part_id, self.chapter_id, self.part_idx, self.chapter_idx, &doc))
//     }
//
//     pub fn map_doc<O, F>(self, f: F) -> anyhow::Result<ConfigItemRef<'a, O>> where F: Fn(&Document<D>) -> anyhow::Result<O> {
//         let doc = Document {
//             id: self.doc.id.clone(),
//             format: self.doc.format.clone(),
//             path: self.doc.path.clone(),
//             content: f(self.doc)?,
//         };
//         Ok(ConfigItem::new(self.part_id, self.chapter_id, self.part_idx, self.chapter_idx, &doc))
//     }
// }


impl<D: Clone + Default> FromIterator<ConfigItem<D>> for Config<D> {
    fn from_iter<T: IntoIterator<Item=ConfigItem<D>>>(iter: T) -> Self {
        // let mut index = it.next().unwrap().doc;
        let mut index: Document<D> = Document {
            id: "".to_string(),
            format: Format::Markdown,
            path: Default::default(),
            content: Arc::new(D::default())
        };

        let mut parts: Vec<Part<D>> = vec![];

        let mut last_chapter = 0;

        for item in iter {
            match item.part_idx.unwrap() {
                0 => index = item.doc,
                part_idx => {
                    let part_id = item.part_id.unwrap();
                    match item.chapter_idx.unwrap() {
                        0 => parts.push(Part { id: part_id, index: item.doc, chapters: vec![] }),
                        chapter_idx => {
                            let chapter_id = item.chapter_id.unwrap();

                            if last_chapter == chapter_idx {
                                parts.last_mut().unwrap().chapters.last_mut().unwrap().documents.push(item.doc);
                            } else {
                                parts.last_mut().unwrap().chapters.push(Chapter {
                                    id: chapter_id,
                                    index: item.doc,
                                    documents: vec![],
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
            index: index,
            content: parts
        }
    }
}
//
// impl<'a, D> Iterator for ConfigIterator<&'a D> where D: Clone {
//     type Item = ConfigItem<&'a D>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         match self.part_pos {
//             0 => { // Config index
//                 self.part_pos += 1;
//                 Some(ConfigItem::new(None, None, Some(0), None, self.config.index.clone()))
//             }
//             part_idx if part_idx <= self.config.content.len() => {
//                 let part = &self.config.content[part_idx - 1];
//
//                 let current_chapter_pos = self.chapter_pos;
//
//
//
//                 match current_chapter_pos {
//                     0 => { // Part index
//                         if part.chapters.len() == 0 {
//                             self.part_pos += 1;
//                         } else {
//                             self.chapter_pos += 1;
//                         }
//                         Some(ConfigItem::new(Some(part.id.clone()), None, Some(part_idx), Some(0), part.index.clone()))
//                     }
//
//                     chapter_idx => {
//                         let chapter = &part.chapters[chapter_idx - 1];
//
//                         let current_doc_pos = self.doc_pos;
//
//                         if current_doc_pos >= chapter.documents.len() {
//                             if current_chapter_pos >= part.chapters.len() {
//                                 self.part_pos += 1;
//                                 self.chapter_pos = 0;
//                             } else {
//                                 self.chapter_pos += 1;
//                             }
//                             self.doc_pos = 0;
//                         } else {
//                             self.doc_pos += 1;
//                         }
//
//
//
//                         match current_doc_pos {
//                             0 => { // Chapter index
//                                 Some(ConfigItem::new(Some(part.id.clone()), Some(chapter.id.clone()), Some(part_idx), Some(chapter_idx), chapter.index.clone()))
//                             }
//                             doc_pos => {
//                                 Some(ConfigItem::new(Some(part.id.clone()), Some(chapter.id.clone()), Some(part_idx), Some(chapter_idx), chapter.documents[doc_pos - 1].clone()))
//                             }
//                         }
//                     }
//                 }
//             }
//             _ => None
//         }
//     }
// }

impl<D> Iterator for ConfigIterator<D> where D: Clone {
    type Item = ConfigItem<D>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.part_pos {
            0 => { // Config index
                self.part_pos += 1;
                Some(ConfigItem::new(None, None, Some(0), None, self.config.index.clone()))
            }
            part_idx if part_idx <= self.config.content.len() => {
                let part = &self.config.content[part_idx - 1];

                let current_chapter_pos = self.chapter_pos;



                match current_chapter_pos {
                    0 => { // Part index
                        if part.chapters.len() == 0 {
                            self.part_pos += 1;
                        } else {
                            self.chapter_pos += 1;
                        }
                        Some(ConfigItem::new(Some(part.id.clone()), None, Some(part_idx), Some(0), part.index.clone()))
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
                            0 => { // Chapter index
                                Some(ConfigItem::new(Some(part.id.clone()), Some(chapter.id.clone()), Some(part_idx), Some(chapter_idx), chapter.index.clone()))
                            }
                            doc_pos => {
                                Some(ConfigItem::new(Some(part.id.clone()), Some(chapter.id.clone()), Some(part_idx), Some(chapter_idx), chapter.documents[doc_pos - 1].clone()))
                            }
                        }
                    }
                }
            }
            _ => None
        }
    }
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
    pub(crate) content: Arc<C>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config<C> {
    pub project_path: PathBuf,
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

impl<I, O> Transform<Config<O>, I, O> for Config<I> {
    fn transform<F>(&self, f: &F) -> Config<O>
        where
            F: Fn(&Document<I>) -> O,
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
            F: Fn(&Document<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
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

impl<I, O> Transform<Document<O>, I, O> for Document<I> {
    fn transform<F>(&self, f: &F) -> Document<O>
        where
            F: Fn(&Document<I>) -> O,
    {
        Document {
            id: self.id.clone(),
            format: self.format.clone(),
            path: self.path.clone(),
            content: Arc::new(f(self)),
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
            content: Arc::new(f(self, part, chapter)),
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
            content: Arc::new(()),
        })
    }
}

const EXT: [&str; 2] = ["md", "ipynb"];

fn extension_in(extension: &str) -> bool {
    EXT.iter().any(|e| e == &extension)
}

fn index_helper<P: AsRef<Path>, PC: AsRef<Path>>(chapter_dir: &P, content_path: &PC) -> anyhow::Result<Document<()>> {
    let chapter_index_md = chapter_dir.as_ref().join("index.md");
    let chapter_index_ipynb = chapter_dir.as_ref().join("index.ipynb");
    let chapter_index = if chapter_index_md.is_file() {
        chapter_index_md
    } else {
        chapter_index_ipynb
    };

    Document::new(chapter_index.strip_prefix(content_path.as_ref())?)
}

impl Chapter<()> {
    fn new<P: AsRef<Path>, PC: AsRef<Path>>(chapter_dir: P, content_path: PC) -> anyhow::Result<Self> {
        let section_dir = chapter_dir.as_ref();

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
            .map(|entry| Document::new(entry.path().strip_prefix(content_path.as_ref())?))
            .collect::<anyhow::Result<Vec<Document<()>>>>()?;

        let index_doc = index_helper(&chapter_dir, &content_path);

        Ok(Chapter {
            id: chapter_id(chapter_dir).ok_or(anyhow!("Can't get chapter id"))?,
            index: index_doc?,
            documents,
        })
    }
}

impl Part<()> {
    fn new<P: AsRef<Path>, PC: AsRef<Path>>(dir: P, content_path: PC) -> anyhow::Result<Self> {
        let part_folder = chapter_id(&dir).ok_or(anyhow!("Can't get part id"))?;
        // let part_dir = dir.as_ref().join(&part_folder);

        let chapters = match fs::read_dir(&dir) {
            Ok(dir) => {
                let paths = dir
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.metadata().map(|meta| meta.is_dir()).unwrap());

                paths
                    .map(|entry| Chapter::new(entry.path(), content_path.as_ref()))
                    .collect::<anyhow::Result<Vec<Chapter<()>>>>()?
            }
            Err(_) => Vec::new(),
        };

        Ok(Part {
            id: part_folder,
            index: index_helper(&dir, &content_path)?,
            chapters,
        })
    }
}

impl Config<()> {
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
