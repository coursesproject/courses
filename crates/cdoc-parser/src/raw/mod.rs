mod parser;

pub use parser::*;
use std::collections::HashMap;

use crate::code_ast::types::CodeContent;
use crate::common::Span;
use cowstr::CowStr;
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};

#[derive(Debug, PartialEq, Default)]
pub struct RawDocument {
    pub(crate) src: Vec<ElementInfo>,
    pub(crate) input: CowStr,
    pub(crate) meta: Option<CowStr>,
    pub(crate) references: HashMap<CowStr, Reference>,
}

impl RawDocument {
    pub fn new(input: &str) -> Self {
        Self {
            src: vec![],
            input: CowStr::from(input),
            meta: None,
            references: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Reference {
    Math(CowStr),
    Code(CowStr),
    Command(CowStr, Vec<Parameter>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Element {
    Markdown(CowStr),
    Special(Option<CowStr>, Special),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Special {
    Math {
        inner: CowStr,
        is_block: bool,
    },
    CodeInline {
        inner: CowStr,
    },
    CodeBlock {
        lvl: usize,
        inner: CodeContent,
        attributes: Vec<CowStr>,
    },
    Command {
        function: CowStr,
        parameters: Vec<Parameter>,
        body: Option<Vec<ElementInfo>>,
    },
    Verbatim {
        inner: CowStr,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodeAttr {
    pub key: Option<CowStr>,
    pub value: CowStr,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ElementInfo {
    pub(crate) element: Element,
    pub(crate) span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub key: Option<CowStr>,
    pub value: Value,
    pub span: Span,
}

impl Parameter {
    pub fn with_value(value: Value, pos: Span) -> Self {
        Self {
            key: None,
            value,
            span: pos,
        }
    }

    pub fn with_key<C: Into<CowStr>>(key: C, value: Value, pos: Span) -> Self {
        Self {
            key: Some(key.into()),
            value,
            span: pos,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Flag(CowStr),
    Content(Vec<ElementInfo>),
    String(CowStr),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Flag(k) => format!("Flag: {k}"),
            Value::Content(_) => "Content".to_string(),
            Value::String(s) => s.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Child {
    pub elem: Special,
    pub span: Span,
    pub label: Option<CowStr>,
    pub identifier: usize,
}

pub struct ComposedMarkdown {
    pub src: CowStr,
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
                    writer.write_all(s.as_bytes()).unwrap();
                }
                Element::Special(label, inner) => {
                    let idx = code_idx + command_idx + math_idx + extra_idx;

                    let identifier = match inner {
                        Special::Math { .. } => {
                            math_idx += 1;
                            math_idx - 1
                        }
                        Special::CodeBlock { lvl, .. } => {
                            if lvl > 1 {
                                code_idx += 1;
                                code_idx - 1
                            } else {
                                extra_idx += 1;
                                0
                            }
                        }
                        Special::Command { .. } => {
                            command_idx += 1;
                            command_idx - 1
                        }
                        Special::Verbatim { .. } => {
                            extra_idx += 1;
                            0
                        }
                        Special::CodeInline { .. } => {
                            extra_idx += 1;
                            0
                        }
                    };

                    children.push(Child {
                        elem: inner,
                        span: elem.span,
                        label,
                        identifier,
                    });
                    write!(&mut writer, "<elem-{}>", idx).unwrap() // Important: Trailing space is necessary as it is eaten by the parser
                }
            }
        }

        ComposedMarkdown {
            src: CowStr::from(String::from_utf8(writer.into_inner().unwrap()).unwrap()),
            children,
        }
    }
}
