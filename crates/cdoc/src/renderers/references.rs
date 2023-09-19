use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Block, Parameter, Reference, Value};
use cdoc_parser::code_ast::types::CodeContent;
use cdoc_parser::raw::CodeAttr;
use cdoc_parser::PosInfo;
use std::collections::HashMap;

pub struct ReferenceVisitor {
    pub(crate) references: HashMap<String, Reference>,
}

impl ReferenceVisitor {
    pub fn new() -> Self {
        ReferenceVisitor {
            references: Default::default(),
        }
    }
}

impl AstVisitor for ReferenceVisitor {
    fn visit_code_block(
        &mut self,
        label: &mut Option<String>,
        _source: &mut CodeContent,
        tags: &mut Vec<CodeAttr>,
        _display_cell: &mut bool,
        _global_idx: &mut usize,
        _pos: &mut PosInfo,
    ) -> anyhow::Result<()> {
        if let Some(label) = label {
            self.references.insert(
                label.to_string(),
                Reference {
                    obj_type: "code".to_string(),
                    attr: Default::default(), // TODO: Attrs
                    num: 0,
                },
            );
        }
        Ok(())
    }

    fn visit_command(
        &mut self,
        function: &mut String,
        id: &mut Option<String>,
        parameters: &mut Vec<Parameter>,
        body: &mut Option<Vec<Block>>,
        _pos: &mut PosInfo,
        _global_idx: &mut usize,
    ) -> anyhow::Result<()> {
        let params = parameters
            .iter()
            .filter_map(|p| {
                p.key.as_ref().and_then(|k| match &p.value {
                    Value::String(s) => Some((k.to_string(), s.clone())),
                    _ => None,
                })
            })
            .collect();
        if let Some(id) = id {
            self.references.insert(
                id.to_string(),
                Reference {
                    obj_type: function.to_string(),
                    attr: params,
                    num: 0,
                },
            );
        }
        if let Some(body) = body {
            self.walk_vec_block(body)?;
        }
        Ok(())
    }

    // TODO: Math block
}
