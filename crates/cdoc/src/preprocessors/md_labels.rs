use crate::parser::ParserSettings;
use crate::preprocessors::{AstPreprocessorConfig, Error, PreprocessorContext, Processor};
use cdoc_base::node::visitor::NodeVisitor;
use cdoc_base::node::{Compound, Node};

use cdoc_base::document::Document;
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
    ) -> anyhow::Result<Box<dyn Processor>> {
        Ok(Box::new(MdLabels))
    }
}

impl Processor for MdLabels {
    fn name(&self) -> String {
        todo!()
    }

    fn process(&mut self, mut input: Document<Vec<Node>>) -> Result<Document<Vec<Node>>, Error> {
        self.walk_elements(&mut input.content)?;
        Ok(input)
    }
}

impl Display for MdLabels {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// TODO: Implement

impl NodeVisitor for MdLabels {
    fn visit_compound(&mut self, node: &mut Compound) -> anyhow::Result<()> {
        if node.type_id == "heading" {
            if let Some(label) = node.children.iter_mut().find(|i| {
                i.get_compound()
                    .map(|c| c.type_id == "label")
                    .unwrap_or_default()
            }) {
                let label_val = label.get_compound().unwrap();
                node.id = label_val.id.clone();
                *label = Node::Plain(String::new());
            }
        }

        Ok(())
    }
}
