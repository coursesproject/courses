mod code_block;
mod types;

use crate::ast::{AstVisitor, CodeAttributes, Inline};
use crate::notebook::CellOutput;
use std::path::PathBuf;

use crate::document::DocumentMetadata;
use anyhow::{anyhow, Result};

use crate::scripting::code_block::{CellOutputData, CellOutputError, CellOutputStream};
use code_block::ScriptCodeBlock;
use rhai::{exported_module, CustomType, Engine, EvalAltResult, Func, Scope, TypeBuilder};

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

fn get_url(url: &str) -> Result<Response, Box<EvalAltResult>> {
    reqwest::blocking::get(url)
        .map(|r| r.into())
        .map_err(|e| e.to_string().into())
}

pub struct ScriptedVisitor {
    engine: Engine,
    ast: rhai::AST,
    state: Scope<'static>,
}

impl ScriptedVisitor {
    pub fn new(project_dir: &PathBuf, script: &str) -> Result<Self> {
        let mut engine = Engine::new();
        engine.set_max_expr_depths(1000, 1000);
        engine.build_type::<Response>();
        engine.build_type::<DocumentMetadata>();
        engine.build_type::<ScriptCodeBlock>();
        engine.build_type::<CellOutputStream>();
        engine.build_type::<CellOutputData>();
        engine.build_type::<CellOutputError>();
        engine.register_fn("get_url", get_url);

        let module = exported_module!(types::rhai_inline_type);
        engine.register_global_module(module.into());

        engine
            .definitions()
            .with_headers(true)
            .include_standard_packages(false)
            .write_to_dir(project_dir.join(".cache"))?;

        let ast = engine.compile(script)?;
        let mut state = Scope::new();

        engine
            .run_ast_with_scope(&mut state, &ast)
            .map_err(|e| anyhow!(e.to_string()))?;
        // engine
        //     .call_fn(&mut state, &ast, "init", ())
        //     .map_err(|e| anyhow!(e.to_string()))?;

        Ok(ScriptedVisitor { ast, engine, state })
    }

    pub fn finalize(&mut self, meta: &DocumentMetadata) -> Result<()> {
        match self
            .engine
            .call_fn::<()>(&mut self.state, &self.ast, "finalize", (meta.clone(),))
        {
            Ok(_) => Ok(()),
            Err(e) => match *e {
                EvalAltResult::ErrorFunctionNotFound(_, _) => Ok(()),
                EvalAltResult::ErrorRuntime(value, _) => {
                    Err(anyhow!(format!("script error: {}", value)))
                }
                _ => Err(anyhow!(format!("{}", e))),
            },
        }
    }
}

impl AstVisitor for ScriptedVisitor {
    fn visit_inline(&mut self, inline: &mut Inline) -> Result<()> {
        let mut scope = &mut self.state;

        match self.engine.call_fn::<Inline>(
            &mut scope,
            &self.ast,
            "visit_inline",
            (inline.clone(),),
        ) {
            Ok(v) => {
                *inline = v;
                Ok(())
            }
            Err(e) => match *e {
                EvalAltResult::ErrorFunctionNotFound(_, _) => Ok(()),
                EvalAltResult::ErrorRuntime(value, _) => {
                    Err(anyhow!(format!("script error: {}", value)))
                }
                _ => Err(anyhow!(format!("{}", e))),
            },
        }?;

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
        let mut scope = &mut self.state;

        let block = ScriptCodeBlock::new(source, reference, attr, tags, outputs, *display_cell);

        match self.engine.call_fn::<ScriptCodeBlock>(
            &mut scope,
            &self.ast,
            "visit_code_block",
            (block,),
        ) {
            Ok(v) => v.apply_changes(source, reference, attr, tags, outputs, display_cell),
            Err(e) => match *e {
                EvalAltResult::ErrorFunctionNotFound(_, _) => Ok(()),
                EvalAltResult::ErrorRuntime(value, _) => Err(anyhow!(format!("{}", value))),
                _ => Err(anyhow!(format!("{}", e))),
            },
        }?;

        Ok(())
    }
}
