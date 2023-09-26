use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{CodeBlock, Command, Math, Reference, Value};

use linked_hash_map::LinkedHashMap;

pub struct ReferenceVisitor {
    pub(crate) references: LinkedHashMap<String, Reference>,
}

impl ReferenceVisitor {
    pub fn new() -> Self {
        ReferenceVisitor {
            references: Default::default(),
        }
    }
}

impl AstVisitor for ReferenceVisitor {
    fn visit_code_block(&mut self, block: &mut CodeBlock) -> anyhow::Result<()> {
        if let Some(label) = &block.label {
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

    fn visit_math(&mut self, math: &mut Math) -> anyhow::Result<()> {
        if let Some(label) = &math.label {
            self.references.insert(
                label.to_string(),
                Reference {
                    obj_type: "equation".to_string(),
                    attr: Default::default(),
                    num: 0,
                },
            );
        }
        Ok(())
    }

    fn visit_command(&mut self, cmd: &mut Command) -> anyhow::Result<()> {
        let params = cmd
            .parameters
            .iter()
            .filter_map(|p| {
                p.key.as_ref().and_then(|k| match &p.value {
                    Value::String(s) => Some((k.clone(), s.clone())),
                    _ => None,
                })
            })
            .collect();
        if let Some(id) = &cmd.label {
            self.references.insert(
                id.to_string(),
                Reference {
                    obj_type: cmd.function.to_string(),
                    attr: params,
                    num: 0,
                },
            );
        }
        if let Some(body) = &mut cmd.body {
            self.walk_vec_block(body)?;
        }
        Ok(())
    }

    // TODO: Math block
}
