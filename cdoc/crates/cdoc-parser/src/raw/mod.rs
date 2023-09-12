mod parser;

pub use parser::*;

use crate::common::PosInfo;
use std::io::{BufWriter, Write};

#[derive(Debug, PartialEq)]
pub struct RawDocument {
    pub(crate) src: Vec<ElementInfo>,
    pub(crate) meta: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Element {
    Markdown(String),
    Extern(Extern),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Extern {
    Math {
        inner: String,
        is_block: bool,
    },
    Code {
        lvl: usize,
        inner: String,
    },
    Command {
        function: String,
        id: Option<String>,
        parameters: Vec<Parameter>,
        body: Option<Vec<ElementInfo>>,
    },
    Verbatim(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ElementInfo {
    pub(crate) element: Element,
    pub(crate) pos: PosInfo,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub key: Option<String>,
    pub value: Value,
    pub span: PosInfo,
}

impl Parameter {
    pub fn with_value(value: Value, span: PosInfo) -> Self {
        Self {
            key: None,
            value,
            span,
        }
    }

    pub fn with_key<C: Into<String>>(key: C, value: Value, span: PosInfo) -> Self {
        Self {
            key: Some(key.into()),
            value,
            span,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Flag(String),
    Content(Vec<ElementInfo>),
    String(String),
}

#[derive(Clone)]
pub struct Child {
    pub elem: Extern,
    pub pos: PosInfo,
    pub identifier: usize,
}

pub struct ComposedMarkdown {
    pub src: String,
    pub children: Vec<Child>,
}

impl From<Vec<ElementInfo>> for ComposedMarkdown {
    fn from(value: Vec<ElementInfo>) -> Self {
        let mut writer = BufWriter::new(Vec::new());
        let mut children = Vec::new();

        let mut code_idx = 0;
        let mut command_idx = 0;

        for elem in value.into_iter() {
            match elem.element {
                Element::Markdown(s) => {
                    writer.write(s.as_bytes()).unwrap();
                }
                Element::Extern(inner) => {
                    let idx = code_idx + command_idx;

                    let identifier = match inner {
                        Extern::Math { .. } => 0,
                        Extern::Code { .. } => {
                            code_idx += 1;
                            code_idx - 1
                        }
                        Extern::Command { .. } => {
                            command_idx += 1;
                            command_idx - 1
                        }
                        Extern::Verbatim(_) => 0,
                    };

                    children.push(Child {
                        elem: inner,
                        pos: elem.pos,
                        identifier,
                    });
                    write!(&mut writer, "_+elem-{}+_", idx).unwrap()
                }
            }
        }

        ComposedMarkdown {
            src: String::from_utf8(writer.into_inner().unwrap()).unwrap(),
            children,
        }
    }
}
