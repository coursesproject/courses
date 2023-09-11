#[derive(Debug)]
pub struct RawDocument {
    pub(crate) src: Vec<Element>,
    pub(crate) meta: Option<String>,
}

#[derive(Debug)]
pub enum Element {
    Markdown(String),
    Verbatim(String),
    Math {
        inner: String,
        is_block: bool,
    },
    Code {
        inner: String,
    },
    Call {
        function: String,
        id: Option<String>,
        parameters: Vec<Parameter>,
        body: Option<Vec<Element>>,
    },
}

#[derive(Debug)]
pub struct Parameter {
    key: Option<String>,
    value: Value,
}

#[derive(Debug)]
pub enum Value {
    Flag(String),
    Content(Vec<Element>),
    String(String),
}
