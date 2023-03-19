use crate::ast::iterators::InnerContent;
use crate::ast::{AEvent, ATag, Ast, Block, CodeAttributes, Inline};
use pulldown_cmark::{Event, Tag};

impl<'a> FromIterator<Event<'a>> for Ast {
    fn from_iter<T: IntoIterator<Item = Event<'a>>>(iter: T) -> Self {
        let iter = iter.into_iter();

        let mut inners = vec![InnerContent::Blocks(Vec::new())];

        for event in iter {
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
                            .push(Block::Paragraph(inner.into_inlines())),
                        Tag::Heading(lvl, id, classes) => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::Heading {
                                lvl,
                                id: id.map(|s| s.to_string()),
                                classes: classes.into_iter().map(|s| s.to_string()).collect(),
                                inner: inner.into_inlines(),
                            }),
                        Tag::BlockQuote => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::BlockQuote(inner.into_inlines())),
                        Tag::CodeBlock(_) => {
                            inners
                                .last_mut()
                                .unwrap()
                                .blocks_mut()
                                .push(Block::CodeBlock {
                                    source: inner
                                        .into_inlines()
                                        .iter()
                                        .map(|item| item.to_string())
                                        .collect(),
                                    reference: None,
                                    attr: CodeAttributes::default(),
                                    outputs: vec![],
                                });
                        }
                        Tag::List(idx) => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::List(idx, inner.into_blocks())),
                        Tag::Item => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::ListItem(inner.into_blocks())),
                        Tag::Emphasis => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Emphasis(inner.into_inlines())),
                        Tag::Strong => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Strong(inner.into_inlines())),
                        Tag::Strikethrough => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Strikethrough(inner.into_inlines())),
                        Tag::Link(tp, url, title) => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Link(tp, url.to_string(), title.to_string())),
                        Tag::Image(tp, url, title) => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Image(tp, url.to_string(), title.to_string())),
                        _ => {} // TODO: Unreachable
                    }
                }
                Event::Html(s) => {
                    inners
                        .last_mut()
                        .unwrap()
                        .push_inline(Inline::Html(s.into_string()));
                }
                other => {
                    let inner = match other {
                        Event::Text(s) => Inline::Text(s.into_string()),
                        Event::Code(s) => Inline::Code(s.into_string()),
                        Event::SoftBreak => Inline::SoftBreak,
                        Event::HardBreak => Inline::HardBreak,
                        Event::Rule => Inline::Rule,
                        _ => unreachable!(),
                    };

                    if let Some(c) = inners.last_mut() {
                        c.push_inline(inner)
                    }
                }
            }
        }
        let blocks = inners.remove(0).into_blocks();
        Ast(blocks)
    }
}

impl FromIterator<AEvent> for Ast {
    fn from_iter<T: IntoIterator<Item = AEvent>>(iter: T) -> Self {
        let iter = iter.into_iter();

        let mut inners = vec![InnerContent::Blocks(Vec::new())];

        for event in iter {
            match event {
                AEvent::Start(t) => match t {
                    ATag::Paragraph
                    | ATag::Heading(_, _, _)
                    | ATag::BlockQuote
                    | ATag::CodeBlock(_)
                    | ATag::TableHead
                    | ATag::TableRow
                    | ATag::TableCell
                    | ATag::Emphasis
                    | ATag::Strong
                    | ATag::Strikethrough
                    | ATag::Image(_, _, _) => inners.push(InnerContent::Inlines(Vec::new())),
                    ATag::Link(_, _, _) => inners.push(InnerContent::Inlines(Vec::new())),
                    ATag::List(_) | ATag::Item | ATag::Table(_) | ATag::FootnoteDefinition(_) => {
                        inners.push(InnerContent::Blocks(Vec::new()))
                    }
                },
                AEvent::End(t) => {
                    let inner = inners.pop().expect("No inner content");
                    match t {
                        ATag::Paragraph => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::Paragraph(inner.into_inlines())),
                        ATag::Heading(lvl, id, classes) => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::Heading {
                                lvl,
                                id,
                                classes,
                                inner: inner.into_inlines(),
                            }),
                        ATag::BlockQuote => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::BlockQuote(inner.into_inlines())),
                        ATag::CodeBlock(_) => {
                            inners
                                .last_mut()
                                .unwrap()
                                .blocks_mut()
                                .push(Block::CodeBlock {
                                    source: inner
                                        .into_inlines()
                                        .iter()
                                        .map(|item| item.to_string())
                                        .collect(),
                                    reference: None,
                                    attr: CodeAttributes::default(),
                                    outputs: vec![],
                                });
                        }
                        ATag::List(idx) => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::List(idx, inner.into_blocks())),
                        ATag::Item => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::ListItem(inner.into_blocks())),
                        ATag::Emphasis => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Emphasis(inner.into_inlines())),
                        ATag::Strong => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Strong(inner.into_inlines())),
                        ATag::Strikethrough => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Strikethrough(inner.into_inlines())),
                        ATag::Link(tp, url, title) => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Link(tp, url.to_string(), title.to_string())),
                        ATag::Image(tp, url, title) => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Image(tp, url.to_string(), title.to_string())),
                        _ => {} // TODO: Unreachable
                    }
                }
                AEvent::Html(s) => {
                    inners.last_mut().unwrap().push_inline(Inline::Html(s));
                }
                other => {
                    let inner = match other {
                        AEvent::Text(s) => Inline::Text(s),
                        AEvent::Code(s) => Inline::Code(s),
                        AEvent::SoftBreak => Inline::SoftBreak,
                        AEvent::HardBreak => Inline::HardBreak,
                        AEvent::Rule => Inline::Rule,
                        _ => unreachable!(),
                    };

                    if let Some(c) = inners.last_mut() {
                        c.push_inline(inner)
                    }
                }
            }
        }
        let blocks = inners.remove(0).into_blocks();
        Ast(blocks)
    }
}
