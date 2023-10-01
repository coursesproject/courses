use crate::parser::ParserSettings;
use crate::preprocessors::{AstPreprocessor, AstPreprocessorConfig, Error, PreprocessorContext};
use cdoc_base::node::visitor::ElementVisitor;
use cdoc_base::node::{Element, Node};
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Ast, Block, Inline};
use cdoc_parser::document::Document;
use cowstr::CowStr;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MdLabelsConfig;

pub struct MdLabels;

#[typetag::serde(name = "md_labels")]
impl AstPreprocessorConfig for MdLabelsConfig {
    fn build(
        &self,
        _ctx: &PreprocessorContext,
        _settings: &ParserSettings,
    ) -> anyhow::Result<Box<dyn AstPreprocessor>> {
        Ok(Box::new(MdLabels))
    }
}

impl AstPreprocessor for MdLabels {
    fn name(&self) -> String {
        todo!()
    }

    fn process(
        &mut self,
        mut input: Document<Vec<Element>>,
    ) -> Result<Document<Vec<Element>>, Error> {
        self.walk_elements(&mut input.content)?;
        Ok(input)
    }
}

impl Display for MdLabels {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl ElementVisitor for MdLabels {
    fn visit_node(&mut self, node: &mut Node) -> anyhow::Result<()> {
        if node.type_id == "heading" {}

        Ok(())
    }
}

impl AstVisitor for MdLabels {
    fn visit_block(&mut self, block: &mut Block) -> anyhow::Result<()> {
        if let Block::Heading { id, inner, .. } = block {
            if let Some(cmd) = inner.iter_mut().find(|i| match i {
                Inline::Command(c) => c.function.as_str() == "label",
                _ => false,
            }) {
                if let Inline::Command(label) = cmd {
                    *id = label.label.clone();
                }
                *cmd = Inline::Text(CowStr::new());
            } else {
                *id = Some(CowStr::from(nanoid!()));
            }
        }

        self.walk_block(block)
    }
}
