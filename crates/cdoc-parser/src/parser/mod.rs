use crate::ast::*;

use crate::raw;
use crate::raw::{ArgumentVal, Child, ComposedMarkdown, Special};
use anyhow::anyhow;
use cowstr::ToCowStr;
use lazy_static::lazy_static;
use pulldown_cmark::{Event, HeadingLevel, Parser as MdParser, Tag};
use regex::Regex;
use std::str::FromStr;

pub(crate) enum InnerContent {
    Blocks(Vec<Block>),
    Inlines(Vec<Inline>),
}

impl InnerContent {
    pub(crate) fn into_blocks(self) -> Vec<Block> {
        if let InnerContent::Blocks(b) = self {
            b
        } else {
            panic!("Expected blocks")
        }
    }

    pub(crate) fn into_inlines(self) -> Vec<Inline> {
        if let InnerContent::Inlines(i) = self {
            i
        } else {
            panic!("Expected inlines")
        }
    }

    pub(crate) fn blocks_mut(&mut self) -> anyhow::Result<&mut Vec<Block>> {
        if let InnerContent::Blocks(b) = self {
            Ok(b)
        } else {
            Err(anyhow!("Expected block element"))
        }
    }

    #[allow(unused)]
    fn inlines_mut(&mut self) -> anyhow::Result<&mut Vec<Inline>> {
        if let InnerContent::Inlines(i) = self {
            Ok(i)
        } else {
            Err(anyhow!("Expected inline element"))
        }
    }

    pub(crate) fn push_inline(&mut self, item: Inline) {
        match self {
            InnerContent::Blocks(b) => b.push(Block::Plain(vec![item])),
            InnerContent::Inlines(i) => i.push(item),
        }
    }
}

impl From<raw::ArgumentVal> for Value {
    fn from(value: raw::ArgumentVal) -> Self {
        match value {
            raw::ArgumentVal::Flag(f) => Value::Flag(f),
            raw::ArgumentVal::Content(c) => Value::Content(ComposedMarkdown::from(c).into()),
            raw::ArgumentVal::String(s) => Value::String(s),
            ArgumentVal::Int(i) => Value::String(i.to_string().into()),
            ArgumentVal::Float(f) => Value::String(f.to_string().into()),
        }
    }
}

impl From<raw::Parameter> for Parameter {
    fn from(value: raw::Parameter) -> Self {
        Parameter {
            key: value.key,
            value: value.value.into(),
            span: value.span,
        }
    }
}

// impl From<raw::Reference> for Reference {
//     fn from(value: raw::Reference) -> Self {
//         match value {
//             raw::Reference::Math(s) => Reference::Math(s),
//             raw::Reference::Code(s) => Reference::Code(s),
//             raw::Reference::Command(name, val) => Reference::Command {
//                 function: name,
//                 parameters: val.into_iter().map(|p| p.into()).collect(),
//             },
//         }
//     }
// }

impl From<Child> for Inline {
    fn from(value: Child) -> Self {
        match value.elem {
            Special::Math { inner, is_block } => Inline::Math(Math {
                label: value.label,
                source: inner,
                display_block: is_block,
                span: value.span,
            }),
            Special::CodeBlock {
                inner, attributes, ..
            } => Inline::CodeBlock(CodeBlock {
                label: value.label,
                source: inner,
                attributes,
                display_cell: false,
                global_idx: value.identifier,
                span: value.span,
            }),
            Special::Script { src, .. } => Inline::Code(src),
            Special::CodeInline { inner } => Inline::Code(inner),
            Special::Command {
                function,
                parameters,
                body,
            } => {
                let parameters = parameters.into_iter().map(|p| p.into()).collect();
                let body = body.map(|b| ComposedMarkdown::from(b).into());

                Inline::Command(Command {
                    function,
                    label: value.label,
                    parameters,
                    body,
                    span: value.span,
                    global_idx: value.identifier,
                })
            }
            Special::Verbatim { inner } => Inline::Text(inner),
        }
    }
}

lazy_static! {
    static ref r: Regex = Regex::new(r"elem-([0-9]+)").expect("invalid regex expression");
}

impl From<ComposedMarkdown> for Vec<Block> {
    fn from(composed: ComposedMarkdown) -> Self {
        let parser: MdParser = MdParser::new(&composed.src);

        let mut inners = vec![InnerContent::Blocks(Vec::new())];

        for event in parser {
            match event {
                Event::Start(t) => match t {
                    Tag::Paragraph
                    | Tag::Heading(_, _, _)
                    | Tag::BlockQuote
                    | Tag::CodeBlock(_)
                    | Tag::TableHead
                    | Tag::TableRow
                    | Tag::TableCell
                    | Tag::Emphasis
                    | Tag::Strong
                    | Tag::Strikethrough
                    | Tag::Image(_, _, _) => inners.push(InnerContent::Inlines(Vec::new())),
                    Tag::Link(_, _, _) => inners.push(InnerContent::Inlines(Vec::new())),
                    Tag::List(_) | Tag::Item | Tag::Table(_) | Tag::FootnoteDefinition(_) => {
                        inners.push(InnerContent::Blocks(Vec::new()))
                    }
                },
                Event::End(t) => {
                    let inner = inners.pop().expect("No inner content");
                    match t {
                        Tag::Paragraph => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .expect("for paragraph")
                            .push(Block::Paragraph(inner.into_inlines())),
                        Tag::Heading(lvl, id, classes) => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .expect("for heading")
                            .push(Block::Heading {
                                lvl: heading_to_lvl(lvl),
                                id: id.map(|s| s.into()),
                                classes: classes.into_iter().map(|s| s.into()).collect(),
                                inner: inner.into_inlines(),
                            }),
                        Tag::BlockQuote => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .expect("for blockquote")
                            .push(Block::BlockQuote(inner.into_inlines())),
                        Tag::List(idx) => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .expect("for list")
                            .push(Block::List(idx, inner.into_blocks())),
                        Tag::Item => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .expect("for item")
                            .push(Block::ListItem(inner.into_blocks())),
                        Tag::Emphasis => {
                            let src = inner.into_inlines();

                            inners
                                .last_mut()
                                .unwrap()
                                .push_inline(Inline::Styled(src, Style::Emphasis))
                        }
                        Tag::Strong => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Styled(inner.into_inlines(), Style::Strong)),
                        Tag::Strikethrough => inners.last_mut().unwrap().push_inline(
                            Inline::Styled(inner.into_inlines(), Style::Strikethrough),
                        ),
                        Tag::Link(tp, url, alt) => {
                            inners.last_mut().unwrap().push_inline(Inline::Link(
                                tp,
                                url.to_cowstr(),
                                alt.to_cowstr(),
                                inner.into_inlines(),
                            ))
                        }
                        Tag::Image(tp, url, alt) => {
                            inners.last_mut().unwrap().push_inline(Inline::Image(
                                tp,
                                url.to_cowstr(),
                                alt.to_cowstr(),
                                inner.into_inlines(),
                            ))
                        }
                        _ => {} // TODO: Implement rest
                    }
                }
                Event::Html(src) => {
                    let is_insert = r.captures(src.as_ref()).and_then(|c| c.get(1));

                    if let Some(match_) = is_insert {
                        let idx = usize::from_str(match_.as_str()).unwrap();
                        let elem = composed.children[idx].clone();
                        inners.last_mut().unwrap().push_inline(elem.into());
                    } else {
                        inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Html(src.to_cowstr()));
                    }
                }
                other => {
                    let inner = match other {
                        Event::Text(s) => Inline::Text(s.to_cowstr()),
                        Event::Code(s) => Inline::Code(s.to_cowstr()),
                        Event::SoftBreak => Inline::SoftBreak,
                        Event::HardBreak => Inline::HardBreak,
                        Event::Rule => Inline::Rule,
                        _ => unreachable!(),
                    };

                    let c = inners.last_mut().unwrap();
                    c.push_inline(inner);
                }
            }
        }
        let b = inners.remove(0).into_blocks();
        b.clone()
    }
}

fn heading_to_lvl(value: HeadingLevel) -> u8 {
    match value {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use crate::ast;
    use crate::ast::Block::ListItem;
    use crate::ast::{Block, Command, Inline, Math, Parameter, Style, Value};
    use crate::code_ast::types::{CodeContent, CodeElem};
    use crate::common::Span;
    use crate::raw::{parse_to_doc, ComposedMarkdown, Element, ElementInfo, Special};

    use pulldown_cmark::LinkType;

    #[test]
    fn simple_command() {
        let stuff = vec![
            ElementInfo {
                element: Element::Markdown("regular stuff ".into()),
                span: Span::new(0, 0),
            },
            ElementInfo {
                element: Element::Special(
                    None,
                    Special::Command {
                        function: "func".into(),
                        parameters: vec![],
                        body: Some(vec![ElementInfo {
                            element: Element::Markdown("x".into()),
                            span: Span::new(0, 0),
                        }]),
                    },
                ),
                span: Span::new(0, 0),
            },
        ];

        let composed = ComposedMarkdown::from(stuff);
        let doc = Vec::from(composed);

        let expected = vec![Block::Paragraph(vec![
            Inline::Text("regular stuff ".into()),
            Inline::Command(Command {
                function: "func".into(),
                label: None,
                parameters: vec![],
                body: Some(vec![Block::Paragraph(vec![Inline::Text("x".into())])]),
                span: Span::new(0, 0),
                global_idx: 0,
            }),
        ])];

        assert_eq!(expected, doc);
    }

    #[test]
    fn markdown_elements() {
        let input = include_str!("../../resources/tests/markdown_elems.md");
        let input_doc = parse_to_doc(input).expect("rawdoc parse error");
        let composed = ComposedMarkdown::from(input_doc.src);
        let output_doc = Vec::from(composed);

        let expected = vec![
            Block::Heading {
                lvl: 1,
                id: None,
                classes: vec![],
                inner: vec![Inline::Text("Heading".into())],
            },
            Block::Heading {
                lvl: 2,
                id: None,
                classes: vec![],
                inner: vec![Inline::Text("Subheading".into())],
            },
            Block::List(
                None,
                vec![
                    ListItem(vec![Block::Plain(vec![Inline::Text(
                        "unordered list".into(),
                    )])]),
                    ListItem(vec![Block::Plain(vec![Inline::Text("item 2".into())])]),
                ],
            ),
            Block::List(
                Some(1),
                vec![
                    ListItem(vec![Block::Plain(vec![Inline::Text(
                        "ordered list".into(),
                    )])]),
                    ListItem(vec![Block::Plain(vec![Inline::Text("item 2".into())])]),
                ],
            ),
            Block::Paragraph(vec![
                Inline::Link(
                    LinkType::Inline,
                    "path/is/here".into(),
                    "".into(),
                    vec![Inline::Text("link".into())],
                ),
                Inline::SoftBreak,
                Inline::Image(
                    LinkType::Inline,
                    "path/is/here".into(),
                    "".into(),
                    vec![Inline::Text("image".into())],
                ),
            ]),
            Block::Paragraph(vec![
                Inline::Styled(vec![Inline::Text("emph".into())], Style::Emphasis),
                Inline::SoftBreak,
                Inline::Styled(vec![Inline::Text("strong".into())], Style::Strong),
            ]),
            Block::Plain(vec![Inline::Code("code inline".into())]),
            Block::Plain(vec![Inline::CodeBlock(ast::CodeBlock {
                label: None,
                source: CodeContent {
                    blocks: vec![CodeElem::Src("\ncode block\n\n".to_string())],
                    meta: Default::default(),
                    hash: 8014072465408005981,
                },

                display_cell: false,
                global_idx: 0,
                span: Span::new(180, 198),
                attributes: vec![],
            })]),
            Block::Plain(vec![Inline::Math(Math {
                label: None,
                source: "math inline".into(),
                display_block: false,
                span: Span::new(200, 213),
            })]),
            Block::Plain(vec![Inline::Math(Math {
                label: None,
                source: "\nmath block\n".into(),
                display_block: true,
                span: Span::new(215, 231),
            })]),
        ];

        assert_eq!(expected, output_doc);
    }

    #[test]
    fn commands() {
        let input = include_str!("../../resources/tests/commands.md");
        let input_doc = parse_to_doc(input).expect("rawdoc parse error");
        let composed = ComposedMarkdown::from(input_doc.src);
        let output_doc = Vec::from(composed);

        let expected = vec![
            Block::Plain(vec![Inline::Command(Command {
                function: "func".into(),
                label: None,
                parameters: vec![],
                body: None,
                span: Span::new(0, 5),
                global_idx: 0,
            })]),
            Block::Plain(vec![Inline::Command(Command {
                function: "func_param".into(),
                label: None,
                parameters: vec![
                    Parameter {
                        key: None,
                        value: Value::String("p1".into()),
                        span: Span::new(19, 21),
                    },
                    Parameter {
                        key: Some("x".into()),
                        value: Value::String("p2".into()),
                        span: Span::new(23, 27),
                    },
                ],
                body: None,
                span: Span::new(7, 28),

                global_idx: 1,
            })]),
            Block::Plain(vec![Inline::Command(Command {
                function: "func_body".into(),
                label: None,
                parameters: vec![],
                body: Some(vec![Block::Paragraph(vec![Inline::Text(
                    "hello there".into(),
                )])]),
                span: Span::new(30, 55),
                global_idx: 2,
            })]),
            Block::Plain(vec![Inline::Command(Command {
                function: "func_all".into(),
                label: None,
                parameters: vec![
                    Parameter {
                        key: None,
                        value: Value::String("p1".into()),
                        span: Span::new(67, 69),
                    },
                    Parameter {
                        key: Some("x".into()),
                        value: Value::String("p2".into()),
                        span: Span::new(71, 75),
                    },
                ],
                body: Some(vec![Block::Paragraph(vec![Inline::Text(
                    "hello there".into(),
                )])]),
                span: Span::new(57, 91),
                global_idx: 3,
            })]),
            Block::Plain(vec![Inline::Command(Command {
                function: "func_inner".into(),
                label: None,
                parameters: vec![],
                body: Some(vec![
                    Block::Plain(vec![Inline::Code("#func".into())]),
                    Block::Plain(vec![Inline::Command(Command {
                        function: "inner".into(),
                        label: None,
                        parameters: vec![],
                        body: Some(vec![Block::Plain(vec![Inline::Math(Math {
                            label: None,
                            source: "math".into(),
                            display_block: false,
                            span: Span::new(122, 128),
                        })])]),
                        span: Span::new(114, 130),
                        global_idx: 0,
                    })]),
                ]),
                span: Span::new(93, 132),
                global_idx: 4,
            })]),
        ];

        assert_eq!(expected, output_doc);
    }
}
