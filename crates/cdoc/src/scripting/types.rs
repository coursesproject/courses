use rhai::plugin::*;

#[allow(non_snake_case, non_upper_case_globals)]
#[export_module]
pub(crate) mod rhai_inline_type {
    use cdoc_parser::ast::{CodeBlock, Command, Math, Style};
    use cdoc_parser::Span;
    use cowstr::CowStr;
    use pulldown_cmark::LinkType;
    use rhai::{Array, Dynamic};

    pub type Inline = cdoc_parser::ast::Inline;

    pub fn Text(value: CowStr) -> Inline {
        Inline::Text(value)
    }
    pub fn Styled(value: Vec<Inline>, style: Style) -> Inline {
        Inline::Styled(value, style)
    }

    pub fn Code(value: CowStr) -> Inline {
        Inline::Code(value)
    }

    pub const SoftBreak: Inline = Inline::SoftBreak;
    pub const HardBreak: Inline = Inline::HardBreak;
    pub const Rule: Inline = Inline::Rule;

    pub fn Image(link_type: LinkType, url: CowStr, alt: CowStr, inner: Vec<Inline>) -> Inline {
        Inline::Image(link_type, url, alt, inner)
    }

    pub fn Link(link_type: LinkType, url: CowStr, alt: CowStr, inner: Vec<Inline>) -> Inline {
        Inline::Link(link_type, url, alt, inner)
    }

    pub fn Html(value: CowStr) -> Inline {
        Inline::Html(value)
    }

    pub fn Math(label: Option<CowStr>, source: CowStr, display_block: bool, pos: Span) -> Inline {
        Inline::Math(Math {
            label,
            source,
            display_block,
            span: pos,
        })
    }

    pub fn Shortcode(value: Command) -> Inline {
        Inline::Command(value)
    }

    #[rhai_fn(global, get = "value", pure)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn get_value(value: &mut Inline) -> Array {
        match value {
            Inline::Text(v) => vec![v.clone().as_str().into()] as Array,
            Inline::Styled(i, s) => vec![i.clone().into(), Dynamic::from(s.clone())] as Array,
            Inline::Code(v) => vec![v.clone().as_str().into()] as Array,
            Inline::CodeBlock(CodeBlock {
                label,
                source,
                attributes: tags,
                display_cell,
                global_idx,
                span: pos,
            }) => vec![
                Dynamic::from(label.clone()),
                Dynamic::from(source.clone()),
                Dynamic::from(tags.clone()),
                Dynamic::from(*display_cell),
                Dynamic::from(*global_idx),
                Dynamic::from(pos.clone()),
            ] as Array,
            Inline::SoftBreak => vec![] as Array,
            Inline::HardBreak => vec![] as Array,
            Inline::Rule => vec![] as Array,
            Inline::Image(t, u, a, i) => vec![
                Dynamic::from(*t),
                u.clone().as_str().into(),
                a.clone().as_str().into(),
                i.clone().into(),
            ] as Array,
            Inline::Link(t, u, a, i) => vec![
                Dynamic::from(*t),
                u.clone().as_str().into(),
                a.clone().as_str().into(),
                i.clone().into(),
            ] as Array,
            Inline::Html(v) => vec![v.clone().as_str().into()] as Array,
            Inline::Math(Math {
                source,
                display_block,
                ..
            }) => vec![source.clone().as_str().into(), (*display_block).into()] as Array,
            Inline::Command(c) => vec![Dynamic::from(c.clone())] as Array,
        }
    }

    #[rhai_fn(global, get = "type", pure)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn get_type(value: &mut Inline) -> String {
        match value {
            Inline::Text(_) => "Text".to_string(),
            Inline::Styled(_, _) => "Styled".to_string(),
            Inline::Code(_) => "Code".to_string(),
            Inline::CodeBlock(CodeBlock { .. }) => "CodeBlock".to_string(),
            Inline::SoftBreak => "SoftBreak".to_string(),
            Inline::HardBreak => "HardBreak".to_string(),
            Inline::Rule => "Rule".to_string(),
            Inline::Image(_, _, _, _) => "Image".to_string(),
            Inline::Link(_, _, _, _) => "Link".to_string(),
            Inline::Html(_) => "Html".to_string(),
            Inline::Math(Math { .. }) => "Math".to_string(),
            Inline::Command(_) => "Command".to_string(),
        }
    }
}
