use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::ast::{Ast, AstVisitor, CodeAttributes};
use crate::document::Document;
use crate::notebook::CellOutput;
use crate::parser::ParserSettings;
use crate::parsers::split::parse_code_string;
use crate::processors::{AstPreprocessor, AstPreprocessorConfig, Error, PreprocessorContext};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExercisesConfig;

#[typetag::serde(name = "exercises")]
impl AstPreprocessorConfig for ExercisesConfig {
    fn build(
        &self,
        _ctx: &PreprocessorContext,
        settings: &ParserSettings,
    ) -> anyhow::Result<Box<dyn AstPreprocessor>> {
        Ok(Box::new(Exercises {
            include_solutions: settings.solutions,
        }))
    }
}

#[derive(Debug, Default)]
pub struct Exercises {
    include_solutions: bool,
}

impl AstVisitor for Exercises {
    fn visit_code_block(
        &mut self,
        source: &mut String,
        _reference: &mut Option<String>,
        _attr: &mut CodeAttributes,
        _tags: &mut Option<Vec<String>>,
        _outputs: &mut Vec<CellOutput>,
        _display_cell: &mut bool,
    ) -> anyhow::Result<()> {
        let res = parse_code_string(source.clone().as_ref())?;

        let (pc, solution) = res.split();
        let out_string = if self.include_solutions { solution } else { pc };
        *source = out_string
            .strip_suffix('\n')
            .unwrap_or(&out_string)
            .to_string();
        Ok(())
    }
}

impl AstPreprocessor for Exercises {
    fn name(&self) -> String {
        "Exercises".to_string()
    }

    fn process(&mut self, mut input: Document<Ast>) -> Result<Document<Ast>, Error> {
        self.walk_ast(&mut input.content)?;
        Ok(input)
    }
}

impl Display for Exercises {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
