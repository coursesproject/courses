use crate::config::Format;
use anyhow::Result;
use std::collections::HashMap;

use dyn_clone::DynClone;

use cowstr::CowStr;

use minijinja::value::{Object, Value};
use std::fmt::{Debug, Display, Formatter};
use std::io::Write;
use std::sync::Arc;

use crate::parser::ParserSettings;
use crate::renderers::extensions::RenderExtension;
use crate::renderers::references::ReferenceVisitor;
use cdoc_base::node::Node;

use crate::templates::new::NewTemplateManager;
use cdoc_base::document::Document;
use cdoc_base::node::visitor::NodeVisitor;
use cdoc_parser::notebook::NotebookMeta;

pub mod base;
pub mod extensions;
pub mod json;
pub mod notebook;
mod references;

/// Type alias used to specify that the string is a renderer output.
pub type RenderResult = CowStr;

/// Context that is passed to the render functions.
#[derive(Debug, Clone)]
pub struct RenderContext {
    pub templates: Arc<NewTemplateManager>,
    /// Extra arguments (this type is essentially a wrapped HashMap)
    pub extra_args: HashMap<String, Value>,
    /// For syntax highlighting using Syntect
    // pub syntax_set: &'a SyntaxSet,
    // theme: &'a Theme,
    pub notebook_output_meta: NotebookMeta,
    pub format: Box<dyn Format>,
    pub parser_settings: ParserSettings,
    // pub references: LinkedHashMap<String, Reference>,
    // pub references_by_type: HashMap<String, Vec<(String, Reference)>>,
}

impl Display for RenderContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Object for RenderContext {}

impl RenderContext {
    pub fn new(
        templates: Arc<NewTemplateManager>,
        mut extra_args: HashMap<String, Value>,
        // syntax_set: &'a SyntaxSet,
        // theme: &'a Theme,
        doc: &mut Document<Vec<Node>>,
        notebook_output_meta: NotebookMeta,
        format: Box<dyn Format>,
        parser_settings: ParserSettings,
    ) -> Result<Self> {
        // let mut parameter_resolution = ParameterResolution { templates };
        // parameter_resolution.walk_ast(&mut doc.content.blocks)?;
        //
        let mut ref_visit = ReferenceVisitor::new(&templates);
        ref_visit.walk_elements(&mut doc.content)?;
        // let rbt = references_by_type(&mut ref_visit.references);
        //
        // extra_args.insert("refs", &ref_visit.references);
        // extra_args.insert("refs_by_type", &rbt);

        // TODO: Add defs again
        // extra_args.insert("defs", &templates.definitions);
        // println!("refs {:?}", &ref_visit.references);

        extra_args.insert(
            "refs".to_string(),
            Value::from_serializable(&ref_visit.reference_map()),
        );
        extra_args.insert(
            "refs_by_type".to_string(),
            Value::from_serializable(&ref_visit.references),
        );
        // println!("{:?}", &ctx.references);
        // args.insert("refs_by_type", &ctx.references_by_type);

        Ok(RenderContext {
            templates,
            extra_args,
            // syntax_set,
            // theme,
            notebook_output_meta,
            format,
            parser_settings,
            // references: LinkedHashMap::new(), //ref_visit.references,
            // references_by_type: HashMap::new(), //rbt,
        })
    }
}

// pub fn references_by_type(
//     refs: &mut LinkedHashMap<String, Reference>,
// ) -> HashMap<String, Vec<(String, Reference)>> {
//     let mut type_map = HashMap::new();
//     for (id, reference) in refs {
//         type_map
//             .entry(reference.type_id.to_string())
//             .or_insert(vec![])
//             .push((id.to_string(), reference.clone()));
//
//         reference.num = type_map.get(&reference.type_id).unwrap().len();
//     }
//     type_map
// }

#[typetag::serde]
pub trait RendererConfig: DynClone + Debug + Send + Sync {
    fn build(&self, extensions: Vec<Box<dyn RenderExtension>>)
        -> Result<Box<dyn DocumentRenderer>>;
}

dyn_clone::clone_trait_object!(RendererConfig);

/// Trait used for rendering a whole document. The trait is used for configuring custom formats in
/// the courses project.
pub trait DocumentRenderer {
    fn render_doc<'a>(
        &mut self,
        doc: &Document<Vec<Node>>,
        ctx: &RenderContext,
    ) -> Result<Document<RenderResult>>;
}

// dyn_clone::clone_trait_object!(DocumentRenderer);

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

pub struct RenderedParam {
    pub key: Option<CowStr>,
    pub value: CowStr,
}
