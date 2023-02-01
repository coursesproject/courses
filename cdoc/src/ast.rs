use pulldown_cmark::{Alignment, CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, Tag};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum ACodeBlockKind {
    Indented,
    Fenced(String),
}

#[derive(Clone, Debug)]
pub enum ATag {
    Paragraph,
    Heading(HeadingLevel, Option<String>, Vec<String>),
    BlockQuote,
    CodeBlock(ACodeBlockKind),
    List(Option<u64>),
    Item,
    FootnoteDefinition(String),
    Table(Vec<Alignment>),
    TableHead,
    TableRow,
    TableCell,
    Emphasis,
    Strong,
    Strikethrough,
    Link(LinkType, String, String),
    Image(LinkType, String, String),
}

#[derive(Clone, Debug)]
pub enum AEvent {
    Start(ATag),
    End(ATag),
    Text(String),
    Code(String),
    Html(String),
    FootnoteReference(String),
    SoftBreak,
    HardBreak,
    Rule,
    TaskListMarker(bool),
}

impl<'a> From<CodeBlockKind<'a>> for ACodeBlockKind {
    fn from(c: CodeBlockKind<'a>) -> Self {
        match c {
            CodeBlockKind::Indented => ACodeBlockKind::Indented,
            CodeBlockKind::Fenced(s) => ACodeBlockKind::Fenced(s.to_string()),
        }
    }
}

impl From<ACodeBlockKind> for CodeBlockKind<'static> {
    fn from(c: ACodeBlockKind) -> Self {
        match c {
            ACodeBlockKind::Indented => CodeBlockKind::Indented,
            ACodeBlockKind::Fenced(s) => CodeBlockKind::Fenced(CowStr::Boxed(s.into_boxed_str())),
        }
    }
}

impl<'a> From<Tag<'a>> for ATag {
    fn from(t: Tag<'a>) -> Self {
        match t {
            Tag::Paragraph => ATag::Paragraph,
            Tag::Heading(lvl, opt, attr) => ATag::Heading(
                lvl,
                opt.map(|s| s.to_string()),
                attr.iter().map(|s| s.to_string()).collect(),
            ),
            Tag::BlockQuote => ATag::BlockQuote,
            Tag::CodeBlock(kind) => ATag::CodeBlock(kind.into()),
            Tag::List(n) => ATag::List(n),
            Tag::Item => ATag::Item,
            Tag::FootnoteDefinition(s) => ATag::FootnoteDefinition(s.to_string()),
            Tag::Table(al) => ATag::Table(al),
            Tag::TableHead => ATag::TableHead,
            Tag::TableRow => ATag::TableRow,
            Tag::TableCell => ATag::TableCell,
            Tag::Emphasis => ATag::Emphasis,
            Tag::Strong => ATag::Strong,
            Tag::Strikethrough => ATag::Strikethrough,
            Tag::Link(typ, url, alt) => ATag::Link(typ, url.to_string(), alt.to_string()),
            Tag::Image(typ, url, alt) => ATag::Image(typ, url.to_string(), alt.to_string()),
        }
    }
}

impl<'a> From<&'a ATag> for Tag<'a> {
    fn from(t: &'a ATag) -> Self {
        match t {
            ATag::Paragraph => Tag::Paragraph,
            ATag::Heading(lvl, id, _) => {
                Tag::Heading(lvl.clone(), id.as_ref().map(|s| s.as_str()), vec![])
            }
            ATag::BlockQuote => Tag::BlockQuote,
            ATag::CodeBlock(c) => Tag::CodeBlock(c.clone().into()),
            ATag::List(n) => Tag::List(n.clone()),
            ATag::Item => Tag::Item,
            ATag::FootnoteDefinition(s) => {
                Tag::FootnoteDefinition(CowStr::Boxed(s.clone().into_boxed_str()))
            }
            ATag::Table(align) => Tag::Table(align.clone()),
            ATag::TableHead => Tag::TableHead,
            ATag::TableRow => Tag::TableRow,
            ATag::TableCell => Tag::TableCell,
            ATag::Emphasis => Tag::Emphasis,
            ATag::Strong => Tag::Strong,
            ATag::Strikethrough => Tag::Strikethrough,
            ATag::Link(typ, url, alt) => Tag::Link(
                typ.clone(),
                CowStr::Boxed(url.clone().into_boxed_str()),
                CowStr::Boxed(alt.clone().into_boxed_str()),
            ),
            ATag::Image(typ, url, alt) => Tag::Image(
                typ.clone(),
                CowStr::Boxed(url.clone().into_boxed_str()),
                CowStr::Boxed(alt.clone().into_boxed_str()),
            ),
        }
    }
}

impl<'a> From<Event<'a>> for AEvent {
    fn from(e: Event) -> Self {
        match e {
            Event::Start(t) => AEvent::Start(t.into()),
            Event::End(t) => AEvent::End(t.into()),
            Event::Text(s) => AEvent::Text(s.to_string()),
            Event::Code(s) => AEvent::Code(s.to_string()),
            Event::Html(s) => AEvent::Html(s.to_string()),
            Event::FootnoteReference(s) => AEvent::FootnoteReference(s.to_string()),
            Event::SoftBreak => AEvent::SoftBreak,
            Event::HardBreak => AEvent::HardBreak,
            Event::Rule => AEvent::Rule,
            Event::TaskListMarker(set) => AEvent::TaskListMarker(set),
        }
    }
}

impl<'a> From<&'a AEvent> for Event<'a> {
    fn from(e: &'a AEvent) -> Self {
        match e {
            AEvent::Start(t) => Event::Start(t.into()),
            AEvent::End(t) => Event::End(t.into()),
            AEvent::Text(s) => Event::Text(CowStr::Borrowed(s.as_str())),
            AEvent::Code(s) => Event::Code(CowStr::Borrowed(s.as_str())),
            AEvent::Html(s) => Event::Html(CowStr::Borrowed(s.as_str())),
            AEvent::FootnoteReference(s) => Event::FootnoteReference(CowStr::Borrowed(s.as_str())),
            AEvent::SoftBreak => Event::SoftBreak,
            AEvent::HardBreak => Event::HardBreak,
            AEvent::Rule => Event::Rule,
            AEvent::TaskListMarker(set) => Event::TaskListMarker(*set),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Inline {
    Text(String),
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    SoftBreak,
    HardBreak,
    Rule,
    Image(LinkType, String, String),
    Link(LinkType, String, String),
    Html(String),
}

fn vec_inline_to_string(vec: &Vec<Inline>) -> String {
    vec.iter().map(|item| item.to_string()).collect()
}

impl ToString for Inline {
    fn to_string(&self) -> String {
        match self {
            Inline::Text(s) => s.clone(),
            Inline::Emphasis(inner) => vec_inline_to_string(inner),
            Inline::Strong(inner) => vec_inline_to_string(inner),
            Inline::Strikethrough(inner) => vec_inline_to_string(inner),
            Inline::Code(s) => s.clone(),
            Inline::SoftBreak => String::default(),
            Inline::HardBreak => String::default(),
            Inline::Rule => String::default(),
            Inline::Html(s) => s.to_string(),
            _ => String::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ast(Vec<Block>);

#[derive(Clone, Debug)]
pub struct CodeAttributes {}

#[derive(Clone, Debug)]
pub enum CodeOutput {
    Image(String),
    Svg(String),
    Json(HashMap<String, Value>),
    Html(String),
    Javascript(String),
}

#[derive(Clone, Debug)]
pub enum Block {
    Heading {
        lvl: HeadingLevel,
        id: Option<String>,
        classes: Vec<String>,
        inner: Vec<Inline>,
    },
    Plain(Inline),
    Paragraph(Vec<Inline>),
    BlockQuote(Vec<Inline>),
    CodeBlock {
        source: String,
        reference: Option<String>,
        attr: CodeAttributes,
        outputs: Vec<CodeOutput>,
    },
    List(Option<u64>, Vec<Block>),
    ListItem(Vec<Block>),
}

fn wrap_events(tag: ATag, mut events: Vec<AEvent>) -> std::vec::IntoIter<AEvent> {
    let mut res = vec![AEvent::Start(tag.clone())];
    res.append(&mut events);
    res.append(&mut vec![AEvent::End(tag)]);
    res.into_iter()
}

fn iter_inlines(inlines: &Vec<Inline>) -> Vec<AEvent> {
    inlines
        .into_iter()
        .flat_map(|i| i.clone().into_iter())
        .collect()
}

fn iter_blocks(blocks: &Vec<Block>) -> Vec<AEvent> {
    blocks
        .into_iter()
        .flat_map(|block| block.clone().into_iter())
        .collect()
}

impl IntoIterator for Inline {
    type Item = AEvent;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Inline::Text(s) => vec![AEvent::Text(s)].into_iter(),
            Inline::Emphasis(inner) => wrap_events(ATag::Emphasis, iter_inlines(&inner)),
            Inline::Strong(inner) => wrap_events(ATag::Strong, iter_inlines(&inner)),
            Inline::Strikethrough(inner) => wrap_events(ATag::Strikethrough, iter_inlines(&inner)),
            Inline::Code(s) => vec![AEvent::Code(s)].into_iter(),
            Inline::SoftBreak => vec![AEvent::SoftBreak].into_iter(),
            Inline::HardBreak => vec![AEvent::HardBreak].into_iter(),
            Inline::Rule => vec![AEvent::Rule].into_iter(),
            Inline::Html(s) => vec![AEvent::Html(s)].into_iter(),
            Inline::Image(tp, url, title) => vec![
                AEvent::Start(ATag::Image(tp, url.clone(), title.clone())),
                AEvent::End(ATag::Image(tp, url, title)),
            ]
            .into_iter(),
            Inline::Link(tp, url, title) => vec![
                AEvent::Start(ATag::Link(tp, url.clone(), title.clone())),
                AEvent::End(ATag::Link(tp, url, title)),
            ]
            .into_iter(),
        }
    }
}

impl IntoIterator for Block {
    type Item = AEvent;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        // id.map(|s| s.as_str()) <> classes.into_iter().map(|s| s.as_str()).collect()
        match self {
            Block::Heading {
                lvl,
                id,
                classes,
                inner,
            } => wrap_events(ATag::Heading(lvl, None, vec![]), iter_inlines(&inner)),
            Block::Paragraph(inner) => wrap_events(ATag::Paragraph, iter_inlines(&inner)),
            Block::Plain(inline) => inline.into_iter(),
            Block::BlockQuote(inner) => wrap_events(ATag::BlockQuote, iter_inlines(&inner)),
            Block::CodeBlock {
                source,
                reference,
                attr,
                outputs,
            } => {
                let string = format!("<pre><code>{}</code></pre>", source);
                vec![AEvent::Html(string)].into_iter()
            }
            Block::List(idx, items) => {
                let item_events = items.into_iter().flat_map(|inner| inner.into_iter());
                let full_iter = vec![AEvent::Start(ATag::List(idx))]
                    .into_iter()
                    .chain(item_events)
                    .chain(vec![AEvent::End(ATag::List(idx))]);
                let v: Vec<AEvent> = full_iter.collect();
                v.into_iter()
                // TODO: Change iter type to dynamic
            }
            Block::ListItem(inner) => wrap_events(ATag::Item, iter_blocks(&inner)),
            // Block::Html(s) => vec![AEvent::Html(s.into_boxed_str().to_string())].into_iter(),
        }
    }
}

impl IntoIterator for Ast {
    type Item = AEvent;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.0.into_iter().flat_map(|block| block.into_iter()))
    }
}

enum InnerContent {
    Blocks(Vec<Block>),
    Inlines(Vec<Inline>),
}

impl InnerContent {
    fn to_blocks(self) -> Vec<Block> {
        if let InnerContent::Blocks(b) = self {
            b
        } else {
            panic!("Expected blocks")
        }
    }

    fn to_inlines(self) -> Vec<Inline> {
        if let InnerContent::Inlines(i) = self {
            i
        } else {
            panic!("Expected inlines")
        }
    }

    fn blocks_mut(&mut self) -> &mut Vec<Block> {
        if let InnerContent::Blocks(b) = self {
            b
        } else {
            panic!("Expected blocks")
        }
    }

    fn inlines_mut(&mut self) -> &mut Vec<Inline> {
        if let InnerContent::Inlines(i) = self {
            i
        } else {
            panic!("Expected inlines")
        }
    }

    fn push_inline(&mut self, item: Inline) {
        match self {
            InnerContent::Blocks(b) => b.push(Block::Plain(item)),
            InnerContent::Inlines(i) => i.push(item),
        }
    }
}

impl<'a> FromIterator<Event<'a>> for Ast {
    fn from_iter<T: IntoIterator<Item = Event<'a>>>(iter: T) -> Self {
        let mut iter = iter.into_iter();

        let mut inners = vec![InnerContent::Blocks(Vec::new())];

        while let Some(event) = iter.next() {
            println!("{:?}", event);
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
                            .push(Block::Paragraph(inner.to_inlines())),
                        Tag::Heading(lvl, id, classes) => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::Heading {
                                lvl,
                                id: id.map(|s| s.to_string()),
                                classes: classes.into_iter().map(|s| s.to_string()).collect(),
                                inner: inner.to_inlines(),
                            }),
                        Tag::BlockQuote => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::BlockQuote(inner.to_inlines())),
                        Tag::CodeBlock(variant) => {
                            let info = match variant {
                                CodeBlockKind::Indented => "".to_string(),
                                CodeBlockKind::Fenced(s) => s.to_string(),
                            };
                            inners
                                .last_mut()
                                .unwrap()
                                .blocks_mut()
                                .push(Block::CodeBlock {
                                    source: inner
                                        .to_inlines()
                                        .iter()
                                        .map(|item| item.to_string())
                                        .collect(),
                                    reference: None,
                                    attr: CodeAttributes {},
                                    outputs: vec![],
                                });
                        }
                        Tag::List(idx) => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::List(idx, inner.to_blocks())),
                        Tag::Item => inners
                            .last_mut()
                            .unwrap()
                            .blocks_mut()
                            .push(Block::ListItem(inner.to_blocks())),
                        Tag::Emphasis => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Emphasis(inner.to_inlines())),
                        Tag::Strong => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Strong(inner.to_inlines())),
                        Tag::Strikethrough => inners
                            .last_mut()
                            .unwrap()
                            .push_inline(Inline::Strikethrough(inner.to_inlines())),
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

                    inners.last_mut().map(|c| c.push_inline(inner));
                }
            }
        }
        let blocks = inners.remove(0).to_blocks();
        Ast(blocks)
    }
}
