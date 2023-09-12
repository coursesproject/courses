#[derive(Debug, PartialEq, Clone)]
pub struct PosInfo {
    pub(crate) input: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl PosInfo {
    pub fn new(input: &str, start: usize, end: usize) -> Self {
        PosInfo {
            input: input.to_string(),
            start,
            end,
        }
    }
}

impl<'a> From<pest::Span<'a>> for PosInfo {
    fn from(value: pest::Span) -> Self {
        PosInfo {
            input: value.get_input().to_string(),
            start: value.start(),
            end: value.end(),
        }
    }
}
