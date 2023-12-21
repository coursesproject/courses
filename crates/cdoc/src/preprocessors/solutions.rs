use crate::parser::ParserSettings;
use crate::preprocessors::{AstPreprocessorConfig, Error, PreprocessorContext, Processor};
use cdoc_base::node::visitor::NodeVisitor;
use cdoc_base::node::{Attribute, Compound, Node};

use cdoc_base::document::Document;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Pointer};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SolutionsConfig {}

#[typetag::serde(name = "solutions")]
impl AstPreprocessorConfig for SolutionsConfig {
    fn build(
        &self,
        ctx: &PreprocessorContext,
        settings: &ParserSettings,
    ) -> anyhow::Result<Box<dyn Processor>> {
        Ok(Box::new(Solutions {}))
    }
}

pub struct Solutions {}

impl Processor for Solutions {
    fn name(&self) -> String {
        "solutions".to_string()
    }

    fn process(&mut self, mut input: Document<Vec<Node>>) -> Result<Document<Vec<Node>>, Error> {
        self.walk_elements(&mut input.content)?;
        Ok(input)
    }
}

impl NodeVisitor for Solutions {
    fn visit_compound(&mut self, node: &mut Compound) -> anyhow::Result<()> {
        if &node.type_id == "code_block" {
            self.parse_content(node)?;
        }

        self.walk_node(node)
    }
}

impl Solutions {
    fn parse_content(&mut self, code_node: &mut Compound) -> anyhow::Result<()> {
        let elements = &mut code_node.children;
        let mut solution = String::new();
        let mut placeholder = String::new();

        for elem in elements {
            match elem {
                Node::Compound(solution_block) => {
                    let inners = &mut solution_block.children;
                    for mut inner in inners {
                        let inner = inner.get_compound().unwrap();
                        let val = inner.children[0].get_plain().unwrap();
                        match inner.type_id.as_str() {
                            "placeholder" => {
                                placeholder.push_str(val);
                            }
                            "solution" => {
                                solution.push_str(val);
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                Node::Plain(src) => {
                    solution.push_str(src);
                    placeholder.push_str(src);
                }
                _ => unreachable!(),
            }
        }

        code_node.attributes.push((
            Some("solution".to_string()),
            Attribute::String(solution.trim().to_string()),
        ));
        code_node.attributes.push((
            Some("placeholder".to_string()),
            Attribute::String(placeholder.trim().to_string()),
        ));
        code_node.children = vec![];

        Ok(())
    }
}

impl Display for Solutions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
