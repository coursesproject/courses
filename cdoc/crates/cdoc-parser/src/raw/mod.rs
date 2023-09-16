mod parser;

pub use parser::*;
use std::collections::HashMap;

use crate::code_ast::types::CodeContent;
use crate::common::PosInfo;
use std::io::{BufWriter, Write};

#[derive(Debug, PartialEq, Default)]
pub struct RawDocument {
    pub(crate) src: Vec<ElementInfo>,
    pub(crate) meta: Option<String>,
    pub(crate) references: HashMap<String, Reference>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Reference {
    Math(String),
    Code(String),
    Command(String, Vec<Parameter>),
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
    CodeInline {
        inner: String,
    },
    CodeBlock {
        lvl: usize,
        inner: CodeContent,
        params: Vec<CodeAttr>,
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
pub struct CodeAttr {
    pub key: Option<String>,
    pub value: String,
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
    pub pos: PosInfo,
}

impl Parameter {
    pub fn with_value(value: Value, pos: PosInfo) -> Self {
        Self {
            key: None,
            value,
            pos,
        }
    }

    pub fn with_key<C: Into<String>>(key: C, value: Value, pos: PosInfo) -> Self {
        Self {
            key: Some(key.into()),
            value,
            pos,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Flag(String),
    Content(Vec<ElementInfo>),
    String(String),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Flag(k) => format!("Flag: {k}"),
            Value::Content(_) => format!("Content"),
            Value::String(s) => s.clone(),
        }
    }
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
        let mut math_idx = 0;
        let mut extra_idx = 0;

        for elem in value.into_iter() {
            match elem.element {
                Element::Markdown(s) => {
                    writer.write(s.as_bytes()).unwrap();
                }
                Element::Extern(inner) => {
                    let idx = code_idx + command_idx + math_idx + extra_idx;

                    let identifier = match inner {
                        Extern::Math { .. } => {
                            math_idx += 1;
                            math_idx - 1
                        }
                        Extern::CodeBlock { lvl, .. } => {
                            if lvl > 1 {
                                code_idx += 1;
                                code_idx - 1
                            } else {
                                extra_idx += 1;
                                0
                            }
                        }
                        Extern::Command { .. } => {
                            command_idx += 1;
                            command_idx - 1
                        }
                        Extern::Verbatim(_) => {
                            extra_idx += 1;
                            0
                        }
                        Extern::CodeInline { .. } => {
                            extra_idx += 1;
                            0
                        }
                    };

                    children.push(Child {
                        elem: inner,
                        pos: elem.pos,
                        identifier,
                    });
                    write!(&mut writer, "<elem-{}>", idx).unwrap()
                }
            }
        }

        ComposedMarkdown {
            src: String::from_utf8(writer.into_inner().unwrap()).unwrap(),
            children,
        }
    }
}
