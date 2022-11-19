pub mod katex;
pub mod shortcode_extender;

use crate::document::DocPos;
use crate::extensions::Error::CodeParseError;
use crate::parsers::split::{format_pest_err, human_errors, parse_code_string, Rule};
use crate::parsers::split_types::CodeTaskDefinition;
use anyhow::Context;
use pest::error::InputLocation;
use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub trait Preprocessor {
    fn process(&self, input: &str) -> Result<String, Box<dyn std::error::Error>>;
}

pub trait ExtensionFactory {
    fn build<'a>(&self) -> Box<dyn Extension<'a>>;
}

pub trait Extension<'a> {
    fn each(&mut self, event: (Event<'a>, DocPos)) -> Result<(Event<'a>, DocPos), Error>;
}

pub struct CodeSplitFactory {}

impl ExtensionFactory for CodeSplitFactory {
    fn build<'a>(&self) -> Box<dyn Extension<'a>> {
        Box::new(CodeSplit::default())
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("code split syntax error at {}: {}", .1, .0)]
    CodeParseError(#[source] pest::error::Error<Rule>, DocPos),
    #[error("could not parse attributes: {}", .0)]
    AttrParseError(#[from] toml::de::Error),
}

#[derive(Debug, Default)]
pub struct CodeSplit {
    code_started: bool,
    pub solution_string: String,
    pub source_def: CodeTaskDefinition,
}

impl CodeSplit {
    pub fn get_source_def(&self) -> &CodeTaskDefinition {
        &self.source_def
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CodeAttrs {
    lang: String,
    #[serde(default = "default_split")]
    perform_split: bool,
}

fn default_split() -> bool {
    false
}

impl<'a> Extension<'a> for CodeSplit {
    fn each(&mut self, event: (Event<'a>, DocPos)) -> Result<(Event<'a>, DocPos), Error> {
        match event.0 {
            Event::Start(tag) => match &tag {
                Tag::CodeBlock(attribute_string) => {
                    // self.code_started = true;
                    // TODO: Find other way to test the attribute string (possibly parse it)
                    if let CodeBlockKind::Fenced(attr_str) = attribute_string {
                        let res = if attr_str.find(",").is_some() {
                            let formatted = attr_str.replace(",", "\n");
                            let attrs: CodeAttrs = toml::from_str(&formatted)?;
                            self.code_started = attrs.perform_split;
                            Ok((
                                Event::Start(Tag::CodeBlock(Fenced(CowStr::Boxed(
                                    attrs.lang.into_boxed_str(),
                                )))),
                                event.1,
                            ))
                        } else {
                            self.code_started = true;
                            Ok((Event::Start(tag), event.1))
                        };
                        return res;
                    }
                    Ok((Event::Start(tag), event.1))
                }
                _ => Ok((Event::Start(tag), event.1)),
            },
            Event::End(tag) => match &tag {
                Tag::CodeBlock(_content) => {
                    self.code_started = false;
                    Ok((Event::End(tag), event.1))
                }
                _ => Ok((Event::End(tag), event.1)),
            },
            Event::Text(txt) => {
                if self.code_started {
                    let res = parse_code_string(txt.as_ref());
                    Ok(res
                        .map(|mut doc| {
                            let (placeholder, solution) = doc.split();
                            self.solution_string.push_str(&solution);
                            self.source_def.blocks.append(&mut doc.blocks);

                            (
                                Event::Text(CowStr::Boxed(placeholder.into_boxed_str())),
                                event.1.clone(),
                            )
                        })
                        .map_err(|e| human_errors(e))
                        .map_err(|e| CodeParseError(e, event.1))?)
                    // match res {
                    //     Ok(mut doc) => {
                    //         let (placeholder, solution) = doc.split();
                    //         self.solution_string.push_str(&solution);
                    //         self.source_def.blocks.append(&mut doc.blocks);
                    //
                    //         Event::Text(CowStr::Boxed(placeholder.into_boxed_str()))
                    //     }
                    //     Err(e) => Event::Html(CowStr::Boxed(
                    //         format!(r#"<div class="alert alert-warning">Split parsing failed: {}</div>"#, format_pest_err(e))
                    //             .into_boxed_str(),
                    //     )),
                    // }
                } else {
                    Ok((Event::Text(txt), event.1))
                }
            }
            _ => Ok(event),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder_old::Builder;
    use crate::cfg::Format;
    use crate::config::Document;

    #[test]
    fn test_code_split() {
        let mut builder =
            Builder::new("resources/test/", vec![Box::new(CodeSplitFactory {})]).unwrap();
        let doc = Document {
            format: Format::Markdown,
            path: "resources/test/code.md".into(),
            meta: None,
        };

        let res = builder.parse_pd(doc).unwrap();
        println!("{:?}", res);
    }
}
