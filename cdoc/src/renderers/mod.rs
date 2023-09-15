use crate::config::Format;
use anyhow::Result;
use std::collections::HashMap;

use dyn_clone::DynClone;

use std::fmt::Debug;
use std::io::Write;

use crate::renderers::references::ReferenceVisitor;
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Ast, Reference};
use cdoc_parser::document::Document;
use cdoc_parser::notebook::NotebookMeta;
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use tera::Context;

use crate::templates::TemplateManager;

pub mod generic;
pub mod json;
pub mod notebook;
mod references;

/// Type alias used to specify that the string is a renderer output.
pub type RenderResult = String;

/// Context that is passed to the render functions.
pub struct RenderContext<'a> {
    /// The document that is being rendered
    pub doc: &'a Document<Ast>,
    pub templates: &'a TemplateManager,
    /// Extra arguments (this type is essentially a wrapped HashMap)
    pub extra_args: Context,
    /// For syntax highlighting using Syntect
    pub syntax_set: &'a SyntaxSet,
    theme: &'a Theme,
    pub notebook_output_meta: &'a NotebookMeta,
    pub format: &'a dyn Format,
    pub references: HashMap<String, Reference>,
    pub references_by_type: HashMap<String, Vec<(String, Reference)>>,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        doc: &'a mut Document<Ast>,
        templates: &'a TemplateManager,
        extra_args: Context,
        syntax_set: &'a SyntaxSet,
        theme: &'a Theme,
        notebook_output_meta: &'a NotebookMeta,
        format: &'a dyn Format,
    ) -> Result<Self> {
        let mut ref_visit = ReferenceVisitor::new();
        ref_visit.walk_ast(&mut doc.content.0)?;
        let rbt = references_by_type(&ref_visit.references);

        Ok(RenderContext {
            doc,
            templates,
            extra_args,
            syntax_set,
            theme,
            notebook_output_meta,
            format,
            references: ref_visit.references,
            references_by_type: rbt,
        })
    }
}

pub fn references_by_type(
    refs: &HashMap<String, Reference>,
) -> HashMap<String, Vec<(String, Reference)>> {
    let mut type_map = HashMap::new();
    for (id, reference) in refs {
        let typ = match reference {
            Reference::Math(_) => "math",
            Reference::Code(_) => "code",
            Reference::Command { function, .. } => &function,
        };

        type_map
            .entry(typ.to_string())
            .or_insert(vec![])
            .push((id.to_string(), reference.clone()));
    }
    type_map
}

/// Trait used for rendering a whole document. The trait is used for configuring custom formats in
/// the courses project.
#[typetag::serde]
pub trait DocumentRenderer: DynClone + Debug + Send + Sync {
    fn render_doc(&mut self, ctx: &RenderContext) -> Result<Document<RenderResult>>;
}

dyn_clone::clone_trait_object!(DocumentRenderer);

/// The base trait that renderers should implement for each type used by [create::ast::Ast].
pub trait RenderElement<T> {
    /// Render the element to a buffer
    fn render(&mut self, elem: &T, ctx: &RenderContext, buf: impl Write) -> Result<()>;

    /// Convenience function for creating a buffer, rendering the element into the buffer, and
    /// returning the result as a string. This is useful when an inner element needs to be rendered
    /// first to be used in an outer element, hence the name.
    fn render_inner(&mut self, elem: &T, ctx: &RenderContext) -> Result<String> {
        let mut buf = Vec::new();
        self.render(elem, ctx, &mut buf)?;
        Ok(String::from_utf8(buf)?)
    }
}

/// Implementation for vectors of elements. Automatically implemented for any type that implements
/// the trait.
impl<T: RenderElement<R>, R> RenderElement<Vec<R>> for T {
    fn render(&mut self, elem: &Vec<R>, ctx: &RenderContext, mut buf: impl Write) -> Result<()> {
        elem.iter().try_for_each(|e| self.render(e, ctx, &mut buf))
    }
}
