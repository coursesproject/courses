use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Block, Parameter, Reference};
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
    fn visit_command(
        &mut self,
        function: &mut String,
        id: &mut Option<String>,
        parameters: &mut Vec<Parameter>,
        body: &mut Option<Vec<Block>>,
        _pos: &mut PosInfo,
        _global_idx: &mut usize,
    ) -> anyhow::Result<()> {
        if let Some(id) = id {
            self.references.insert(
                id.to_string(),
                Reference::Command {
                    function: function.to_string(),
                    parameters: parameters.clone(),
                },
            );
        }
        if let Some(body) = body {
            self.walk_vec_block(body)?;
        }
        Ok(())
    }
}
