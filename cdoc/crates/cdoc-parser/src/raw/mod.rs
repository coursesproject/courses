mod doc;

pub use doc::*;

use pest::Span;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct RawDocument<'a> {
    pub(crate) src: Vec<ElementInfo<'a>>,
    pub(crate) meta: Option<Cow<'a, str>>,
}

#[derive(Debug, PartialEq)]
pub enum Element<'a> {
    Markdown(Cow<'a, str>),
    Verbatim(Cow<'a, str>),
    Math {
        inner: Cow<'a, str>,
        is_block: bool,
    },
    Code {
        lvl: usize,
        inner: Cow<'a, str>,
    },
    Command {
        function: Cow<'a, str>,
        id: Option<Cow<'a, str>>,
        parameters: Vec<Parameter<'a>>,
        body: Option<Vec<ElementInfo<'a>>>,
    },
}

#[derive(Debug, PartialEq)]
pub struct ElementInfo<'a> {
    element: Element<'a>,
    span: Span<'a>,
}

#[derive(Debug, PartialEq)]
pub struct Parameter<'a> {
    pub key: Option<Cow<'a, str>>,
    pub value: Value<'a>,
    pub span: Span<'a>,
}

impl<'a> Parameter<'a> {
    pub fn with_value(value: Value<'a>, span: Span<'a>) -> Self {
        Self {
            key: None,
            value,
            span,
        }
    }

    pub fn with_key<C: Into<Cow<'a, str>>>(key: C, value: Value<'a>, span: Span<'a>) -> Self {
        Self {
            key: Some(key.into()),
            value,
            span,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Flag(Cow<'a, str>),
    Content(Vec<ElementInfo<'a>>),
    String(Cow<'a, str>),
}
