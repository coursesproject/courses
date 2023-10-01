use crate::templates::{TemplateManager, TemplateType};
use anyhow::anyhow;
use cdoc_base::node::visitor::ElementVisitor;
use cdoc_base::node::Node;
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::Command;

pub struct ParameterResolution<'a> {
    pub templates: &'a TemplateManager,
}

impl AstVisitor for ParameterResolution<'_> {
    fn visit_command(&mut self, cmd: &mut Command) -> anyhow::Result<()> {
        for (i, param) in cmd.parameters.iter_mut().enumerate() {
            if param.key.is_none() {
                let def = self
                    .templates
                    .get_template(&cmd.function, TemplateType::Shortcode)?
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
        self.walk_command(&mut cmd.body)
    }
}
