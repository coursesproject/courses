use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::ast::{Ast, AstVisitor, CodeAttributes};
use crate::document::Document;
use crate::notebook::CellOutput;
use crate::parsers::split::parse_code_string;
use crate::processors::{AstPreprocessor, AstPreprocessorConfig, Error, PreprocessorContext};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExercisesConfig;

#[typetag::serde(name = "exercises")]
impl AstPreprocessorConfig for ExercisesConfig {
    fn build(&self, _ctx: &PreprocessorContext) -> anyhow::Result<Box<dyn AstPreprocessor>> {
        Ok(Box::new(Exercises))
    }
}

// #[typetag::serde(name = "code_split")]
// impl EventPreprocessorConfig for ExercisesConfig {
//     fn build(&self, _ctx: &PreprocessorContext) -> anyhow::Result<Box<dyn EventPreprocessor>> {
//         Ok(Box::new(Exercises))
//     }
// }

#[derive(Debug)]
pub struct Exercises;

impl AstVisitor for Exercises {
    fn visit_code_block(
        &mut self,
        source: &mut String,
        _reference: &mut Option<String>,
        _attr: &mut CodeAttributes,
        _outputs: &mut Vec<CellOutput>,
    ) -> anyhow::Result<()> {
        let res = parse_code_string(source.clone().as_ref())?;
        let (pc, _) = res.split();
        *source = pc.strip_suffix("\n").unwrap_or(&pc).to_string();
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

// impl EventPreprocessor for Exercises {
//     fn name(&self) -> String {
//         "Code split".to_string()
//     }
//
//     fn process(&self, input: Document<EventContent>) -> Result<Document<EventContent>, Error> {
//         let mut code_block = false;
//         let mut source = "".to_string();
//         let mut code_attr = String::new();
//
//         let content = input
//             .content
//             .into_iter()
//             .flat_map(|event| match &event {
//                 AEvent::Start(tag) => {
//                     if let ATag::CodeBlock(ACodeBlockKind::Fenced(attr)) = &tag {
//                         code_block = true;
//                         code_attr = attr.to_string();
//                     }
//                     vec![Ok(AEvent::Start(tag.clone()))]
//                 }
//                 AEvent::End(tag) => {
//                     if let ATag::CodeBlock(ACodeBlockKind::Fenced(_)) = tag {
//                         // TODO: Here
//                         let res = parse_code_string(source.clone().as_ref());
//                         code_block = false;
//                         source = String::new();
//                         match res {
//                             Ok(doc) => {
//                                 let (pc, _solution) = doc.split();
//                                 vec![
//                                     Ok(AEvent::Text(pc.trim().to_string())),
//                                     Ok(AEvent::End(tag.clone())),
//                                 ]
//                             }
//                             Err(e) => vec![Err(CodeParseError(human_errors(*e)))],
//                         }
//                     } else {
//                         vec![Ok(event)]
//                     }
//                 }
//                 AEvent::Text(txt) => {
//                     if code_block {
//                         source.push_str(txt.as_ref());
//                         vec![]
//                     } else {
//                         vec![Ok(AEvent::Text(txt.clone()))]
//                     }
//                 }
//                 _ => vec![Ok(event)],
//             })
//             .collect::<Result<Vec<AEvent>, Error>>()?;
//
//         Ok(Document {
//             metadata: input.metadata,
//             variables: input.variables,
//             content,
//         })
//     }
// }

impl Display for Exercises {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
