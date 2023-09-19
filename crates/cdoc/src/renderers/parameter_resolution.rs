use crate::templates::{TemplateManager, TemplateType};
use anyhow::anyhow;
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Block, Parameter};
use cdoc_parser::PosInfo;

pub struct ParameterResolution<'a> {
    pub templates: &'a TemplateManager,
}

impl AstVisitor for ParameterResolution<'_> {
    fn visit_command(
        &mut self,
        function: &mut String,
        _id: &mut Option<String>,
        parameters: &mut Vec<Parameter>,
        _body: &mut Option<Vec<Block>>,
        _pos: &mut PosInfo,
        _global_idx: &mut usize,
    ) -> anyhow::Result<()> {
        for (i, param) in parameters.iter_mut().enumerate() {
            if let None = param.key {
                let def = self
                    .templates
                    .get_template(&function, TemplateType::Shortcode)?
                    .shortcode
                    .unwrap();
                param.key = Some(
                    def.parameters
                        .get(i)
                        .ok_or(anyhow!("Too many arguments"))?
                        .name
                        .clone(),
                );
            }
        }
        Ok(())
    }
}
