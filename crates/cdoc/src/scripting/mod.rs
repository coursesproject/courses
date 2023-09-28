mod code_block;
mod types;

use std::collections::HashMap;
use std::path::Path;

use anyhow::{anyhow, Result};

use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{CodeBlock, Inline};

use cdoc_parser::document::{CodeOutput, Metadata};

use cdoc_parser::code_ast::types::CodeContent;
use code_block::ScriptCodeBlock;
use rhai::{exported_module, CustomType, Engine, EvalAltResult, Scope, TypeBuilder};

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

pub struct ScriptEngine {
    engine: Engine,
    ast: rhai::AST,
}

impl ScriptEngine {
    pub fn new(project_dir: &Path, script: &str) -> Result<Self> {
        let mut engine = Engine::new();
        engine.set_max_expr_depths(1000, 1000);
        engine.build_type::<Response>();
        engine.build_type::<Metadata>();
        engine.build_type::<ScriptCodeBlock>();
        engine.register_fn("get_url", get_url);

        let module = exported_module!(types::rhai_inline_type);
        engine.register_global_module(module.into());

        engine
            .definitions()
            .with_headers(true)
            .include_standard_packages(false)
            .write_to_dir(project_dir.join(".cache"))?;

        let ast = engine.compile(script)?;

        engine.run_ast(&ast).map_err(|e| anyhow!(e.to_string()))?;
        // engine
        //     .call_fn(&mut state, &ast, "init", ())
        //     .map_err(|e| anyhow!(e.to_string()))?;

        Ok(ScriptEngine { ast, engine })
    }
}

pub struct ScriptVisitor<'a> {
    base: &'a mut ScriptEngine,
    state: Scope<'static>,
    code_outputs: &'a mut HashMap<u64, CodeOutput>,
}

impl<'a> ScriptVisitor<'a> {
    pub fn new(base: &'a mut ScriptEngine, code_outputs: &'a mut HashMap<u64, CodeOutput>) -> Self {
        ScriptVisitor {
            base,
            state: Scope::new(),
            code_outputs,
        }
    }

    pub fn finalize(&mut self, meta: &Metadata) -> Result<()> {
        match self.base.engine.call_fn::<()>(
            &mut self.state,
            &self.base.ast,
            "finalize",
            (meta.clone(),),
        ) {
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

impl AstVisitor for ScriptVisitor<'_> {
    fn visit_inline(&mut self, inline: &mut Inline) -> Result<()> {
        match self.base.engine.call_fn::<Inline>(
            &mut self.state,
            &self.base.ast,
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

    fn visit_code_block(&mut self, block: &mut CodeBlock) -> Result<()> {
        if let CodeContent::Parsed { blocks, meta, hash } = &block.source {
            let outputs = self.code_outputs.get_mut(hash);
            let cblock = ScriptCodeBlock::new(
                &block.source,
                &block.attributes,
                &outputs,
                block.display_cell,
                block.global_idx,
                &block.span,
            );

            match self.base.engine.call_fn::<ScriptCodeBlock>(
                &mut self.state,
                &self.base.ast,
                "visit_code_block",
                (cblock,),
            ) {
                Ok(v) => v.apply_changes(
                    &mut block.source,
                    &mut block.attributes,
                    outputs,
                    &mut block.display_cell,
                    &mut block.global_idx,
                ),
                Err(e) => match *e {
                    EvalAltResult::ErrorFunctionNotFound(_, _) => Ok(()),
                    EvalAltResult::ErrorRuntime(value, _) => Err(anyhow!(format!("{}", value))),
                    _ => Err(anyhow!(format!("{}", e))),
                },
            }?;
        }

        Ok(())
    }
}
