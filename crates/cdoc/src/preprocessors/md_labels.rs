use crate::parser::ParserSettings;
use crate::preprocessors::{AstPreprocessor, AstPreprocessorConfig, Error, PreprocessorContext};
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Ast, Block, Inline};
use cdoc_parser::document::Document;
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
        ctx: &PreprocessorContext,
        settings: &ParserSettings,
    ) -> anyhow::Result<Box<dyn AstPreprocessor>> {
        Ok(Box::new(MdLabels))
    }
}

impl AstPreprocessor for MdLabels {
    fn name(&self) -> String {
        todo!()
    }

    fn process(&mut self, mut input: Document<Ast>) -> Result<Document<Ast>, Error> {
        self.walk_ast(&mut input.content.0)?;
        Ok(input)
    }
}

impl Display for MdLabels {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl AstVisitor for MdLabels {
    fn visit_block(&mut self, block: &mut Block) -> anyhow::Result<()> {
        if let Block::Heading { id, inner, .. } = block {
            if let Some(cmd) = inner.iter_mut().find(|i| match i {
                Inline::Command(c) => c.function == "label".to_string(),
                _ => false,
            }) {
                if let Inline::Command(label) = cmd {
                    *id = label.label.clone();
                }
                *cmd = Inline::Text(String::new());
            } else {
                *id = Some(nanoid!());
            }
        }

        self.walk_block(block)
    }
}
