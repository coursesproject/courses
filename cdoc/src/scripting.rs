use crate::ast::{AstVisitor, CodeAttributes, Inline};
use crate::notebook::CellOutput;

use anyhow::{anyhow, Result};
use pulldown_cmark::LinkType;
use rhai::plugin::*;
use rhai::{CustomType, Engine, EvalAltResult, Func, Scope, TypeBuilder};

#[allow(non_snake_case, non_upper_case_globals)]
#[export_module]
mod rhai_inline_type {
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

    #[rhai_fn(global, get = "type", pure)]
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

    #[rhai_fn(global, get = "value", pure)]
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
                Dynamic::from(t.clone()),
                u.clone().into(),
                a.clone().into(),
                i.clone().into(),
            ] as Array,
            Inline::Link(t, u, a, i) => vec![
                Dynamic::from(t.clone()),
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
                display_block.clone().into(),
                trailing_space.clone().into(),
            ] as Array,
            Inline::Shortcode(s) => vec![Dynamic::from(s.clone())] as Array,
        }
    }
}

#[derive(Clone)]
struct Response {
    status: String,
    text: String,
}

impl From<reqwest::blocking::Response> for Response {
    fn from(value: reqwest::blocking::Response) -> Self {
        Response {
            status: value.status().to_string(),
            text: value.text().unwrap(),
        }
    }
}

impl CustomType for Response {
    fn build(mut builder: TypeBuilder<Self>) {
        builder
            .with_name("Response")
            .with_get("status", |s: &mut Self| s.status.clone())
            .with_get("text", |s: &mut Self| s.text.clone());
    }
}

fn get_url(url: &str) -> Response {
    reqwest::blocking::get(url).unwrap().into()
}

pub struct ScriptedVisitor {
    engine: Engine,
    ast: rhai::AST,
}

impl ScriptedVisitor {
    pub fn new(script: &str) -> Result<Self> {
        let mut engine = Engine::new();
        engine.build_type::<Response>();
        engine.register_fn("get_url", get_url);

        let module = exported_module!(rhai_inline_type);
        engine.register_global_module(module.into());

        Ok(ScriptedVisitor {
            ast: engine.compile(script)?,
            engine,
        })
    }
}

impl AstVisitor for ScriptedVisitor {
    fn visit_inline(&mut self, inline: &mut Inline) -> Result<()> {
        let mut scope = Scope::new();

        scope.push("inline", inline.clone());

        match self
            .engine
            .call_fn::<()>(&mut scope, &self.ast, "visit_inline", ())
        {
            Ok(_) => Ok(()),
            Err(e) => match *e {
                EvalAltResult::ErrorFunctionNotFound(_, _) => Ok(()),
                _ => Err(anyhow!(format!("{}", e))),
            },
        }?;

        *inline = scope.get_value::<Inline>("inline").unwrap();

        self.walk_inline(inline)
    }
    fn visit_code_block(
        &mut self,
        source: &mut String,
        reference: &mut Option<String>,
        attr: &mut CodeAttributes,
        tags: &mut Option<Vec<String>>,
        outputs: &mut Vec<CellOutput>,
        display_cell: &mut bool,
    ) -> Result<()> {
        let mut scope = Scope::new();

        scope.push("source", source.clone());
        scope.push("reference", reference.clone());
        scope.push("attr", attr.clone());
        scope.push("tags", tags.clone());
        scope.push("outputs", outputs.clone());
        scope.push("display_cell", display_cell.clone());

        match self
            .engine
            .call_fn::<()>(&mut scope, &self.ast, "visit_code_block", ())
        {
            Ok(_) => Ok(()),
            Err(e) => match *e {
                EvalAltResult::ErrorFunctionNotFound(_, _) => Ok(()),
                _ => Err(anyhow!(format!("{}", e))),
            },
        }?;

        *source = scope.get_value::<String>("source").unwrap();
        *reference = scope.get_value::<Option<String>>("reference").unwrap();
        *attr = scope.get_value::<CodeAttributes>("attr").unwrap();
        *tags = scope.get_value::<Option<Vec<String>>>("tags").unwrap();
        *outputs = scope.get_value::<Vec<CellOutput>>("outputs").unwrap();
        *display_cell = scope.get_value::<bool>("display_cell").unwrap();

        Ok(())
    }
}
