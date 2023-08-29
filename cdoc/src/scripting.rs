use crate::ast::{AstVisitor, CodeAttributes};
use crate::notebook::CellOutput;
use crate::parsers::split_types::Output;
use anyhow::{anyhow, Result};
use rhai::{CustomType, Engine, EvalAltResult, Func, Scope, TypeBuilder};
use std::rc::Rc;
use std::sync::Arc;

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
        engine.register_type::<Response>();
        engine.register_fn("get_url", get_url);

        Ok(ScriptedVisitor {
            ast: engine.compile(script)?,
            engine,
        })
    }
}

impl AstVisitor for ScriptedVisitor {
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

        Ok(())
    }
}
