use crate::ast::*;
use crate::common::PosInfo;
use crate::raw;
use crate::raw::{Child, ComposedMarkdown, Extern, RawDocument};
use anyhow::{anyhow, Context};
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

impl From<raw::Value> for Value {
    fn from(value: raw::Value) -> Self {
        match value {
            raw::Value::Flag(f) => Value::Flag(f),
            raw::Value::Content(c) => Value::Content(ComposedMarkdown::from(c).into()),
            raw::Value::String(s) => Value::String(s),
        }
    }
}

impl From<raw::Parameter> for Parameter {
    fn from(value: raw::Parameter) -> Self {
        Parameter {
            key: value.key,
            value: value.value.into(),
            pos: value.span.into(),
        }
    }
}

impl From<Child> for Inline {
    fn from(value: Child) -> Self {
        match value.elem {
            Extern::Math { inner, is_block } => Inline::Math {
                source: inner,
                display_block: is_block,
                pos: value.pos,
            },
            Extern::Code { lvl, inner } => {
                if lvl == 1 {
                    Inline::Code(inner)
                } else {
                    Inline::CodeBlock {
                        source: inner,
                        tags: None,
                        meta: Default::default(),
                        display_cell: false,
                        global_idx: value.identifier,
                        pos: value.pos,
                    }
                }
            }
            Extern::Command {
                function,
                id,
                parameters,
                body,
            } => {
                let parameters = parameters.into_iter().map(|p| p.into()).collect();
                let body = body.map(|b| ComposedMarkdown::from(b).into());

                Inline::Command(Command {
                    function,
                    id,
                    parameters,
                    body,
                    pos: value.pos,
                    global_idx: value.identifier,
                })
            }
            Extern::Verbatim(s) => Inline::Text(s),
        }
    }
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
                                id: id.map(|s| s.to_string()),
                                classes: classes.into_iter().map(|s| s.to_string()).collect(),
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
                            let r =
                                Regex::new(r"\+elem-([0-9]+)\+").expect("invalid regex expression");

                            let is_insert = src
                                .get(0)
                                .and_then(|elem| {
                                    if let Inline::Text(s) = elem {
                                        Some(s)
                                    } else {
                                        None
                                    }
                                })
                                .and_then(|s| r.captures(s.as_ref()))
                                .and_then(|c| c.get(1));

                            if let Some(match_) = is_insert {
                                let idx = usize::from_str(match_.as_str()).unwrap();
                                let elem = composed.children[idx].clone();
                                inners.last_mut().unwrap().push_inline(elem.into())
                            } else {
                                inners
                                    .last_mut()
                                    .unwrap()
                                    .push_inline(Inline::Styled(src, Style::Emphasis))
                            }
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
                                url.to_string(),
                                alt.to_string(),
                                inner.into_inlines(),
                            ))
                        }
                        Tag::Image(tp, url, alt) => {
                            inners.last_mut().unwrap().push_inline(Inline::Image(
                                tp,
                                url.to_string(),
                                alt.to_string(),
                                inner.into_inlines(),
                            ))
                        }
                        _ => unreachable!(),
                    }
                }
                Event::Html(s) => {
                    inners
                        .last_mut()
                        .unwrap()
                        .push_inline(Inline::Html(s.to_string()));
                }
                other => {
                    let inner = match other {
                        Event::Text(s) => Inline::Text(s.to_string()),
                        Event::Code(s) => Inline::Code(s.to_string()),
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
    use crate::ast::Block::ListItem;
    use crate::ast::{Block, Command, Inline, Parameter, Style, Value};
    use crate::common::PosInfo;
    use crate::raw::{parse_to_doc, ComposedMarkdown, Element, ElementInfo, Extern};
    use pulldown_cmark::LinkType;

    #[test]
    fn simple_command() {
        let stuff = vec![
            ElementInfo {
                element: Element::Markdown("regular stuff ".to_string()),
                pos: PosInfo::new("", 0, 0),
            },
            ElementInfo {
                element: Element::Extern(Extern::Command {
                    function: "func".into(),
                    id: None,
                    parameters: vec![],
                    body: Some(vec![ElementInfo {
                        element: Element::Markdown("x".into()),
                        pos: PosInfo::new("", 0, 0),
                    }]),
                }),
                pos: PosInfo::new("", 0, 0),
            },
        ];

        let composed = ComposedMarkdown::from(stuff);
        let doc = Vec::from(composed);

        let expected = vec![Block::Paragraph(vec![
            Inline::Text("regular stuff ".to_string()),
            Inline::Command(Command {
                function: "func".to_string(),
                id: None,
                parameters: vec![],
                body: Some(vec![Block::Paragraph(vec![Inline::Text("x".to_string())])]),
                pos: PosInfo::new("", 0, 0),
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
                inner: vec![Inline::Text("Heading".to_string())],
            },
            Block::Heading {
                lvl: 2,
                id: None,
                classes: vec![],
                inner: vec![Inline::Text("Subheading".to_string())],
            },
            Block::List(
                None,
                vec![
                    ListItem(vec![Block::Plain(vec![Inline::Text(
                        "unordered list".to_string(),
                    )])]),
                    ListItem(vec![Block::Plain(vec![Inline::Text("item 2".to_string())])]),
                ],
            ),
            Block::List(
                Some(1),
                vec![
                    ListItem(vec![Block::Plain(vec![Inline::Text(
                        "ordered list".to_string(),
                    )])]),
                    ListItem(vec![Block::Plain(vec![Inline::Text("item 2".to_string())])]),
                ],
            ),
            Block::Paragraph(vec![
                Inline::Link(
                    LinkType::Inline,
                    "path/is/here".to_string(),
                    String::new(),
                    vec![Inline::Text("link".to_string())],
                ),
                Inline::SoftBreak,
                Inline::Image(
                    LinkType::Inline,
                    "path/is/here".to_string(),
                    String::new(),
                    vec![Inline::Text("image".to_string())],
                ),
            ]),
            Block::Paragraph(vec![
                Inline::Styled(vec![Inline::Text("emph".to_string())], Style::Emphasis),
                Inline::SoftBreak,
                Inline::Styled(vec![Inline::Text("strong".to_string())], Style::Strong),
            ]),
            Block::Paragraph(vec![Inline::Code("code inline".to_string())]),
            Block::Paragraph(vec![Inline::CodeBlock {
                source: "\ncode block\n".to_string(),
                tags: None,
                meta: Default::default(),
                display_cell: false,
                global_idx: 0,
                pos: PosInfo {
                    input: input.to_string(),
                    start: 180,
                    end: 198,
                },
            }]),
            Block::Paragraph(vec![Inline::Math {
                source: "math inline".to_string(),
                display_block: false,
                pos: PosInfo {
                    input: input.to_string(),
                    start: 200,
                    end: 213,
                },
            }]),
            Block::Paragraph(vec![Inline::Math {
                source: "\nmath block\n".to_string(),
                display_block: true,
                pos: PosInfo {
                    input: input.to_string(),
                    start: 215,
                    end: 231,
                },
            }]),
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
            Block::Paragraph(vec![Inline::Command(Command {
                function: "func".to_string(),
                id: None,
                parameters: vec![],
                body: None,
                pos: PosInfo {
                    input: input.to_string(),
                    start: 0,
                    end: 5,
                },
                global_idx: 0,
            })]),
            Block::Paragraph(vec![Inline::Command(Command {
                function: "func_param".to_string(),
                id: None,
                parameters: vec![
                    Parameter {
                        key: None,
                        value: Value::String("p1".to_string()),
                        pos: PosInfo {
                            input: input.to_string(),
                            start: 19,
                            end: 21,
                        },
                    },
                    Parameter {
                        key: Some("x".to_string()),
                        value: Value::String("p2".to_string()),
                        pos: PosInfo {
                            input: input.to_string(),
                            start: 23,
                            end: 27,
                        },
                    },
                ],
                body: None,
                pos: PosInfo {
                    input: input.to_string(),
                    start: 7,
                    end: 28,
                },
                global_idx: 1,
            })]),
            Block::Paragraph(vec![Inline::Command(Command {
                function: "func_body".to_string(),
                id: None,
                parameters: vec![],
                body: Some(vec![Block::Paragraph(vec![Inline::Text(
                    "hello there".to_string(),
                )])]),
                pos: PosInfo {
                    input: input.to_string(),
                    start: 30,
                    end: 55,
                },
                global_idx: 2,
            })]),
            Block::Paragraph(vec![Inline::Command(Command {
                function: "func_all".to_string(),
                id: None,
                parameters: vec![
                    Parameter {
                        key: None,
                        value: Value::String("p1".to_string()),
                        pos: PosInfo {
                            input: input.to_string(),
                            start: 67,
                            end: 69,
                        },
                    },
                    Parameter {
                        key: Some("x".to_string()),
                        value: Value::String("p2".to_string()),
                        pos: PosInfo {
                            input: input.to_string(),
                            start: 71,
                            end: 75,
                        },
                    },
                ],
                body: Some(vec![Block::Paragraph(vec![Inline::Text(
                    "hello there".to_string(),
                )])]),
                pos: PosInfo {
                    input: input.to_string(),
                    start: 57,
                    end: 91,
                },
                global_idx: 3,
            })]),
            Block::Paragraph(vec![Inline::Command(Command {
                function: "func_inner".to_string(),
                id: None,
                parameters: vec![],
                body: Some(vec![Block::Paragraph(vec![
                    Inline::Code("#func".to_string()),
                    Inline::SoftBreak,
                    Inline::Command(Command {
                        function: "inner".to_string(),
                        id: None,
                        parameters: vec![],
                        body: Some(vec![Block::Paragraph(vec![Inline::Math {
                            source: "math".to_string(),
                            display_block: false,
                            pos: PosInfo {
                                input: input.to_string(),
                                start: 122,
                                end: 128,
                            },
                        }])]),
                        pos: PosInfo {
                            input: input.to_string(),
                            start: 114,
                            end: 130,
                        },
                        global_idx: 0,
                    }),
                ])]),
                pos: PosInfo {
                    input: input.to_string(),
                    start: 93,
                    end: 132,
                },
                global_idx: 4,
            })]),
        ];

        assert_eq!(expected, output_doc);
    }
}
