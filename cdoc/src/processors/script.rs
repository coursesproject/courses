// use crate::parser::ParserSettings;
// use crate::processors::{AstPreprocessor, AstPreprocessorConfig, Error, PreprocessorContext};
// // use crate::scripting::ScriptedVisitor;
// use anyhow::Context;
// use serde::{Deserialize, Serialize};
// use std::fmt::{Display, Formatter};
// use std::fs;
//
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct ScriptConfig {
//     name: String,
// }
//
// pub struct ScriptPreprocessor {
//     name: String,
//     visitor: ScriptedVisitor,
// }
//
// #[typetag::serde(name = "script")]
// impl AstPreprocessorConfig for ScriptConfig {
//     fn build(
//         &self,
//         ctx: &PreprocessorContext,
//         _settings: &ParserSettings,
//     ) -> anyhow::Result<Box<dyn AstPreprocessor>> {
//         let script_path = ctx.project_root.join("scripts").join(&self.name);
//         let script = fs::read_to_string(&script_path)
//             .with_context(|| format!("script not found at {}", script_path.display()))?;
//         Ok(Box::new(ScriptPreprocessor {
//             name: self.name.clone(),
//             visitor: ScriptedVisitor::new(&ctx.project_root, &script)?,
//         }))
//     }
// }
//
// impl AstPreprocessor for ScriptPreprocessor {
//     fn name(&self) -> String {
//         self.name.to_string()
//     }
//
//     fn process(&mut self, mut input: Document<Ast>) -> Result<Document<Ast>, Error> {
//         self.visitor.walk_ast(&mut input.content)?;
//         self.visitor.finalize(&input.metadata)?;
//         Ok(input)
//     }
// }
//
// impl Display for ScriptPreprocessor {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.name())
//     }
// }
