use crate::project::{
    Chapter, ContentItem, ContentItemDescriptor, ItemDescriptor, Part, Project, ProjectItem,
    ProjectIterator,
};
use cdoc::config::InputFormat;
use std::path::PathBuf;
use std::sync::Arc;

// impl<C: Clone + Default> FromIterator<ContentItemDescriptor<C>> for ContentItem<C> {
//     fn from_iter<T: IntoIterator<Item = ContentItemDescriptor<C>>>(iter: T) -> Self {
//         from_vec_helper(&mut iter.into_iter()).unwrap()
//     }
// }
//
// impl<'a, C: Clone + Default> FromIterator<&'a ContentItemDescriptor<C>> for ContentItem<&'a C> {
//     fn from_iter<T: IntoIterator<Item = &'a ContentItemDescriptor<C>>>(iter: T) -> Self {
//         from_iter_helper_ref(&mut iter.into_iter())
//     }
// }

// fn from_iter_helper2<C, I: Iterator<Item = ContentItemDescriptor<C>>>(
//     iter: &mut I,
//     current_path: &mut Vec<String>,
// ) -> ContentItem<C> {
//     let index_item = iter.next().unwrap();
//
//     if item.path.len() == current_path.len() + 1 {
//         // same level
//     } else {
//         // new section
//     }
// }

fn from_iter_helper_ref<'a, C, I: Iterator<Item = &'a ContentItemDescriptor<C>>>(
    iter: &mut I,
) -> ContentItem<&'a C> {
    let section_item = iter.next().unwrap();

    let mut children = Vec::new();
    while let Some(item) = iter.next() {
        if item.doc.id != "index" {
            children.push(ContentItem::Document {
                doc: item.doc.as_ref(),
            });
        } else {
            children.push(from_iter_helper_ref(iter));
        }
    }
    ContentItem::Section {
        id: section_item
            .path
            .get(section_item.path.len() - 2)
            .unwrap()
            .clone(),
        doc: section_item.doc.as_ref(),
        children,
    }
}

impl<D> IntoIterator for Project<D>
where
    D: Clone,
{
    type Item = ItemDescriptor<D>;
    type IntoIter = ProjectIterator<D>;

    fn into_iter(self) -> Self::IntoIter {
        ProjectIterator {
            part_pos: 0,
            chapter_pos: 0,
            doc_pos: 0,
            config: self,
        }
    }
}

impl<D> ItemDescriptor<D> {
    fn new(
        part_id: Option<String>,
        chapter_id: Option<String>,
        part_idx: Option<usize>,
        chapter_idx: Option<usize>,
        doc: ProjectItem<D>,
        doc_idx: Option<usize>,
        files: Option<Vec<PathBuf>>,
    ) -> Self {
        ItemDescriptor {
            part_id,
            chapter_id,
            part_idx,
            chapter_idx,
            doc,
            doc_idx,
            files,
        }
    }

    /// Perform operation on the inner document, then return the result wrapped in a ConfigItem.
    pub fn map<O, F>(self, f: F) -> anyhow::Result<ItemDescriptor<O>>
    where
        F: Fn(&D) -> anyhow::Result<O>,
    {
        let doc = ProjectItem {
            id: self.doc.id,
            format: self.doc.format,
            path: self.doc.path,
            content: Arc::new(f(self.doc.content.as_ref())?),
        };
        Ok(ItemDescriptor::new(
            self.part_id,
            self.chapter_id,
            self.part_idx,
            self.chapter_idx,
            doc,
            self.doc_idx,
            self.files,
        ))
    }

    /// Perform operation on the whole DocumentSpec.
    pub fn map_doc<O, F, E>(self, f: F) -> Result<ItemDescriptor<O>, E>
    where
        F: Fn(ProjectItem<D>) -> Result<O, E>,
    {
        let doc = ProjectItem {
            id: self.doc.id.clone(),
            format: self.doc.format,
            path: self.doc.path.clone(),
            content: Arc::new(f(self.doc)?),
        };
        Ok(ItemDescriptor::new(
            self.part_id,
            self.chapter_id,
            self.part_idx,
            self.chapter_idx,
            doc,
            self.doc_idx,
            self.files,
        ))
    }

    // pub fn get_chapter<T>(&self, config: Config<T>) -> Option<Chapter<T>> {
    //     config.content[self.par]
    // }
}

impl<'a, D: Clone + Default> FromIterator<&'a ItemDescriptor<D>> for Project<&'a D> {
    fn from_iter<T: IntoIterator<Item = &'a ItemDescriptor<D>>>(iter: T) -> Self {
        // let mut index = it.next().unwrap().doc;
        let mut index: Option<ProjectItem<&D>> = None;

        let mut parts: Vec<Part<&D>> = vec![];

        let mut last_chapter = 0;

        for item in iter {
            match item.part_idx.unwrap() {
                0 => index = Some(item.doc.as_ref()),
                _part_idx => {
                    let part_id = item.part_id.clone().unwrap();
                    match item.chapter_idx.unwrap() {
                        0 => {
                            last_chapter = 0;
                            parts.push(Part {
                                id: part_id,
                                index: item.doc.as_ref(),
                                chapters: vec![],
                            })
                        }
                        chapter_idx => {
                            let chapter_id = item.chapter_id.clone().unwrap();

                            if last_chapter == chapter_idx {
                                parts
                                    .last_mut()
                                    .unwrap()
                                    .chapters
                                    .last_mut()
                                    .unwrap()
                                    .documents
                                    .push(item.doc.as_ref());
                            } else {
                                parts.last_mut().unwrap().chapters.push(Chapter {
                                    id: chapter_id,
                                    index: item.doc.as_ref(),
                                    documents: vec![],
                                    files: item.files.clone().expect("No files"),
                                });
                                last_chapter = chapter_idx;
                            }
                        }
                    }
                }
            }
        }

        Project {
            project_path: Default::default(),
            index: index.unwrap(),
            content: parts,
        }
    }
}

/// Collect iterator of ConfigItem into Config (tree structure).
impl<D: Clone + Default> FromIterator<ItemDescriptor<D>> for Project<D> {
    fn from_iter<T: IntoIterator<Item = ItemDescriptor<D>>>(iter: T) -> Self {
        // let mut index = it.next().unwrap().doc;
        let mut index: ProjectItem<D> = ProjectItem {
            id: "".to_string(),
            format: InputFormat::Markdown,
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

        Project {
            project_path: Default::default(),
            index,
            content: parts,
        }
    }
}

impl<D> Iterator for ProjectIterator<D>
where
    D: Clone,
{
    type Item = ItemDescriptor<D>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.part_pos {
            0 => {
                // Config index
                self.part_pos += 1;
                Some(ItemDescriptor::new(
                    None,
                    None,
                    Some(0),
                    None,
                    self.config.index.clone(),
                    None,
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
                        Some(ItemDescriptor::new(
                            Some(part.id.clone()),
                            None,
                            Some(part_idx),
                            Some(0),
                            part.index.clone(),
                            None,
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
                                Some(ItemDescriptor::new(
                                    Some(part.id.clone()),
                                    Some(chapter.id.clone()),
                                    Some(part_idx),
                                    Some(chapter_idx),
                                    chapter.index.clone(),
                                    None,
                                    Some(chapter.files.clone()),
                                ))
                            }
                            doc_pos => Some(ItemDescriptor::new(
                                Some(part.id.clone()),
                                Some(chapter.id.clone()),
                                Some(part_idx),
                                Some(chapter_idx),
                                chapter.documents[doc_pos - 1].clone(),
                                Some(doc_pos - 1),
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
