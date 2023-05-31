use crate::ast::{Ast, Block, CodeAttributes, Inline};
use anyhow::{anyhow, Context};
use pulldown_cmark::{Event, Tag};

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

impl Ast {
    pub(crate) fn make_from_iter<'a, T: IntoIterator<Item = Event<'a>>>(
        iter: T,
    ) -> anyhow::Result<Self> {
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
                            .context("for paragraph")?
                            .push(Block::Paragraph(inner.into_inlines())),
                        Tag::Heading(lvl, id, classes) => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .context("for heading")?
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
                            .context("for blockquote")?
                            .push(Block::BlockQuote(inner.into_inlines())),
                        Tag::CodeBlock(_) => {
                            inners
                                .last_mut()
                                .unwrap()
                                .blocks_mut()
                                .context("for code block")?
                                .push(Block::CodeBlock {
                                    source: inner
                                        .into_inlines()
                                        .iter()
                                        .map(|item| item.to_string())
                                        .collect(),
                                    reference: None,
                                    attr: CodeAttributes::default(),
                                    tags: None,
                                    outputs: vec![],
                                });
                        }
                        Tag::List(idx) => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .context("for list")?
                            .push(Block::List(idx, inner.into_blocks())),
                        Tag::Item => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .context("for item")?
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

                    let c = inners.last_mut().unwrap();
                    c.push_inline(inner);
                }
            }
        }
        let blocks = inners.remove(0).into_blocks();
        Ok(Ast(blocks))
    }
}
