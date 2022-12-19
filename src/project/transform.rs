use crate::project::{Chapter, Part, Project, ProjectItem};
use std::sync::Arc;

/// This is a custom map trait meant for configurations. The structure is preserved and each document
/// is transformed.
pub trait Transform<T, I, O> {
    /// Like map but for a configuration.
    fn transform<F>(&self, f: &F) -> T
    where
        F: Fn(&ProjectItem<I>) -> O;
}

/// Convenience trait for a map-function that also has access to the possible parents of a document.
pub trait TransformParents<T, I, O> {
    /// The inner function receives the parents which are omitted if they don't exist.
    fn transform_parents<F>(&self, f: &F) -> T
    where
        F: Fn(&ProjectItem<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O;
}

impl<I, O> Transform<Chapter<O>, I, O> for Chapter<I> {
    fn transform<F>(&self, f: &F) -> Chapter<O>
    where
        F: Fn(&ProjectItem<I>) -> O,
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
        F: Fn(&ProjectItem<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
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
        F: Fn(&ProjectItem<I>) -> O,
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
        F: Fn(&ProjectItem<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
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

impl<I, O> Transform<Project<O>, I, O> for Project<I> {
    fn transform<F>(&self, f: &F) -> Project<O>
    where
        F: Fn(&ProjectItem<I>) -> O,
    {
        Project {
            project_path: self.project_path.clone(),
            index: self.index.transform(f),
            content: self.content.iter().map(|p| p.transform(f)).collect(),
        }
    }
}

impl<I, O> TransformParents<Project<O>, I, O> for Project<I> {
    fn transform_parents<F>(&self, f: &F) -> Project<O>
    where
        F: Fn(&ProjectItem<I>, Option<&Part<I>>, Option<&Chapter<I>>) -> O,
    {
        Project {
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

impl<I, O> Transform<ProjectItem<O>, I, O> for ProjectItem<I> {
    fn transform<F>(&self, f: &F) -> ProjectItem<O>
    where
        F: Fn(&ProjectItem<I>) -> O,
    {
        ProjectItem {
            id: self.id.clone(),
            format: self.format,
            path: self.path.clone(),
            content: Arc::new(f(self)),
        }
    }
}
