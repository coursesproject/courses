use crate::ast::Ast;
use crate::config::Format;
use anyhow::Result;

use dyn_clone::DynClone;

use std::fmt::Debug;
use std::io::Write;

use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use tera::Context;

use crate::document::Document;
use crate::notebook::NotebookMeta;

use crate::templates::TemplateManager;

pub mod generic;
pub mod json;
pub mod notebook;

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
    pub theme: &'a Theme,
    pub notebook_output_meta: &'a NotebookMeta,
    pub format: &'a dyn Format,
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
