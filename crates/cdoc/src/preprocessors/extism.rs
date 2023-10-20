use crate::parser::ParserSettings;
use crate::preprocessors::{AstPreprocessorConfig, Error, PreprocessorContext, Processor};
use cdoc_base::node::Node;

use cdoc_base::document::Document;
use extism::Plugin;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtismConfig {
    file: PathBuf,
}

#[typetag::serde(name = "extism")]
impl AstPreprocessorConfig for ExtismConfig {
    fn build(
        &self,
        ctx: &PreprocessorContext,
        settings: &ParserSettings,
    ) -> anyhow::Result<Box<dyn Processor>> {
        let wasm = fs::read(&self.file).unwrap();
        let mut plugin = Plugin::create(wasm, [], false).unwrap();
        let name = serde_json::from_slice(plugin.call("name", []).unwrap()).unwrap();
        Ok(Box::new(ExtismProcessor { plugin, name }))
    }
}

pub struct ExtismProcessor {
    plugin: Plugin<'static>,
    name: String,
}

impl Processor for ExtismProcessor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn process(&mut self, input: Document<Vec<Node>>) -> Result<Document<Vec<Node>>, Error> {
        let input = serde_json::to_vec(&input).unwrap();
        let output = self.plugin.call("process", &input).unwrap();
        let output = serde_json::from_slice(output).unwrap();
        Ok(output)
    }
}

impl Display for ExtismProcessor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
