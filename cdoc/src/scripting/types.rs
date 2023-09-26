use crate::document::DocumentMetadata;
use rhai::plugin::*;
use rhai::{CustomType, TypeBuilder};

#[allow(non_snake_case, non_upper_case_globals)]
#[export_module]
pub(crate) mod rhai_inline_type {
    use crate::ast::Shortcode;
    use pulldown_cmark::LinkType;
    use rhai::{Array, Dynamic};

    pub type Inline = crate::ast::Inline;

    pub fn Text(value: String) -> Inline {
        Inline::Text(value)
    }
    pub fn Emphasis(value: Vec<Inline>) -> Inline {
        Inline::Emphasis(value)
    }

    pub fn Strong(value: Vec<Inline>) -> Inline {
        Inline::Strong(value)
    }

    pub fn Strikethrough(value: Vec<Inline>) -> Inline {
        Inline::Strikethrough(value)
    }

    pub fn Code(value: String) -> Inline {
        Inline::Code(value)
    }

    pub const SoftBreak: Inline = Inline::SoftBreak;
    pub const HardBreak: Inline = Inline::HardBreak;
    pub const Rule: Inline = Inline::Rule;

    pub fn Image(link_type: LinkType, url: String, alt: String, inner: Vec<Inline>) -> Inline {
        Inline::Image(link_type, url, alt, inner)
    }

    pub fn Link(link_type: LinkType, url: String, alt: String, inner: Vec<Inline>) -> Inline {
        Inline::Link(link_type, url, alt, inner)
    }

    pub fn Html(value: String) -> Inline {
        Inline::Html(value)
    }

    pub fn Math(source: String, display_block: bool, trailing_space: bool) -> Inline {
        Inline::Math {
            source,
            display_block,
            trailing_space,
        }
    }

    pub fn Shortcode(value: Shortcode) -> Inline {
        Inline::Shortcode(value)
    }

    #[rhai_fn(global, get = "value", pure)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn get_value(value: &mut Inline) -> Array {
        match value {
            Inline::Text(v) => vec![v.clone().into()] as Array,
            Inline::Emphasis(i) => vec![i.clone().into()] as Array,
            Inline::Strong(i) => vec![i.clone().into()] as Array,
            Inline::Strikethrough(i) => vec![i.clone().into()] as Array,
            Inline::Code(v) => vec![v.clone().into()] as Array,
            Inline::SoftBreak => vec![] as Array,
            Inline::HardBreak => vec![] as Array,
            Inline::Rule => vec![] as Array,
            Inline::Image(t, u, a, i) => vec![
                Dynamic::from(*t),
                u.clone().into(),
                a.clone().into(),
                i.clone().into(),
            ] as Array,
            Inline::Link(t, u, a, i) => vec![
                Dynamic::from(*t),
                u.clone().into(),
                a.clone().into(),
                i.clone().into(),
            ] as Array,
            Inline::Html(v) => vec![v.clone().into()] as Array,
            Inline::Math {
                source,
                display_block,
                trailing_space,
            } => vec![
                source.clone().into(),
                (*display_block).into(),
                (*trailing_space).into(),
            ] as Array,
            Inline::Shortcode(s) => vec![Dynamic::from(s.clone())] as Array,
        }
    }

    #[rhai_fn(global, get = "type", pure)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn get_type(value: &mut Inline) -> String {
        match value {
            Inline::Text(_) => "Text".to_string(),
            Inline::Emphasis(_) => "Emphasis".to_string(),
            Inline::Strong(_) => "Strong".to_string(),
            Inline::Strikethrough(_) => "Strikethrough".to_string(),
            Inline::Code(_) => "Code".to_string(),
            Inline::SoftBreak => "SoftBreak".to_string(),
            Inline::HardBreak => "HardBreak".to_string(),
            Inline::Rule => "Rule".to_string(),
            Inline::Image(_, _, _, _) => "Image".to_string(),
            Inline::Link(_, _, _, _) => "Link".to_string(),
            Inline::Html(_) => "Html".to_string(),
            Inline::Math { .. } => "Math".to_string(),
            Inline::Shortcode(_) => "Shortcode".to_string(),
        }
    }
}

impl CustomType for DocumentMetadata {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Metadata")
            .with_get("title", |s: &mut Self| s.title.clone())
            .with_get("draft", |s: &mut Self| s.draft)
            .with_get("exercises", |s: &mut Self| s.exercises)
            .with_get("code_solutions", |s: &mut Self| s.code_solutions)
            .with_get("cell_outputs", |s: &mut Self| s.cell_outputs)
            .with_get("interactive", |s: &mut Self| s.interactive)
            .with_get("editable", |s: &mut Self| s.editable)
            .with_get("hide_sidebar", |s: &mut Self| s.layout.hide_sidebar)
            .with_get("exclude_outputs", |s: &mut Self| s.exclude_outputs.clone())
            .with_get("user_defined", |s: &mut Self| s.user_defined.clone());
    }
}
