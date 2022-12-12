pub mod katex;
pub mod shortcode_extender;

use crate::document::DocPos;
use crate::extensions::Error::CodeParseError;
use crate::parser::{FrontMatter};
use crate::parsers::split::{human_errors, parse_code_string, Rule};
use crate::parsers::split_types::CodeTaskDefinition;
use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub trait Preprocessor {
    fn process(&self, input: &str) -> Result<String, Box<dyn std::error::Error>>;
}

pub trait Extension<'a> {
    fn each(&mut self, event: (Event<'a>, DocPos)) -> Result<(Event<'a>, DocPos), Error>;
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("code split syntax error at {}: {}", .1, .0)]
    CodeParseError(#[source] pest::error::Error<Rule>, DocPos),
    #[error("could not parse attributes: {}", .0)]
    AttrParseError(#[from] toml::de::Error),
}

#[derive(Debug)]
pub struct CodeSplit {
    frontmatter: FrontMatter,
    code_started: bool,
    pub source_buf: Vec<String>,
    pub solution_string: String,
    pub source_def: CodeTaskDefinition,
}

impl CodeSplit {
    pub fn get_source_def(&self) -> &CodeTaskDefinition {
        &self.source_def
    }

    pub fn new(frontmatter: FrontMatter) -> Self {
        CodeSplit {
            frontmatter,
            code_started: false,
            source_buf: Vec::new(),
            solution_string: String::new(),
            source_def: CodeTaskDefinition::default(),
        }
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
        if !self.frontmatter.code_split {
            Ok(event)
        } else {
            match event.0 {
                Event::Start(tag) => match &tag {
                    Tag::CodeBlock(attribute_string) => {
                        // self.code_started = true;
                        // TODO: Find other way to test the attribute string (possibly parse it)
                        if let CodeBlockKind::Fenced(attr_str) = attribute_string {
                            let res = if attr_str.find(",").is_some() {
                                let formatted = attr_str.clone().replace(",", "\n");
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
                        self.source_buf.push(txt.to_string());
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

                    } else {
                        Ok((Event::Text(txt), event.1))
                    }
                }
                _ => Ok(event),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}
