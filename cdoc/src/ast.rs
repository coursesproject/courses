use pulldown_cmark::{Alignment, CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, Tag};

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

impl From<ATag> for Tag<'static> {
    fn from(t: ATag) -> Self {
        match t {
            ATag::Paragraph => Tag::Paragraph,
            ATag::Heading(lvl, _, _) => Tag::Heading(lvl, None, vec![]),
            ATag::BlockQuote => Tag::BlockQuote,
            ATag::CodeBlock(c) => Tag::CodeBlock(c.into()),
            ATag::List(n) => Tag::List(n),
            ATag::Item => Tag::Item,
            ATag::FootnoteDefinition(s) => {
                Tag::FootnoteDefinition(CowStr::Boxed(s.into_boxed_str()))
            }
            ATag::Table(align) => Tag::Table(align),
            ATag::TableHead => Tag::TableHead,
            ATag::TableRow => Tag::TableRow,
            ATag::TableCell => Tag::TableCell,
            ATag::Emphasis => Tag::Emphasis,
            ATag::Strong => Tag::Strong,
            ATag::Strikethrough => Tag::Strikethrough,
            ATag::Link(typ, url, alt) => Tag::Link(
                typ,
                CowStr::Boxed(url.into_boxed_str()),
                CowStr::Boxed(alt.into_boxed_str()),
            ),
            ATag::Image(typ, url, alt) => Tag::Image(
                typ,
                CowStr::Boxed(url.into_boxed_str()),
                CowStr::Boxed(alt.into_boxed_str()),
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

impl From<AEvent> for Event<'static> {
    fn from(e: AEvent) -> Self {
        match e {
            AEvent::Start(t) => Event::Start(t.into()),
            AEvent::End(t) => Event::End(t.into()),
            AEvent::Text(s) => Event::Text(CowStr::Boxed(s.into_boxed_str())),
            AEvent::Code(s) => Event::Code(CowStr::Boxed(s.into_boxed_str())),
            AEvent::Html(s) => Event::Html(CowStr::Boxed(s.into_boxed_str())),
            AEvent::FootnoteReference(s) => {
                Event::FootnoteReference(CowStr::Boxed(s.into_boxed_str()))
            }
            AEvent::SoftBreak => Event::SoftBreak,
            AEvent::HardBreak => Event::HardBreak,
            AEvent::Rule => Event::Rule,
            AEvent::TaskListMarker(set) => Event::TaskListMarker(set),
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
    ListItem(Vec<Inline>),
    SoftBreak,
    HardBreak,
    Rule,
}

#[derive(Clone, Debug)]
pub struct Document(Vec<Block>);

pub struct CodeAttributes {}

#[derive(Clone, Debug)]
pub enum Block {
    Heading {
        lvl: HeadingLevel,
        id: Option<String>,
        classes: Vec<String>,
        inner: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    BlockQuote(Vec<Inline>),
    CodeBlock {
        source: Vec<Inline>,
        reference: Option<String>,
        attr: CodeAttributes,
        outputs: Vec<CodeOutput>,
    },
    List(Option<u64>, Vec<Inline>),
    Html(String),
    Image(LinkType, String, String),
}

fn wrap_events<'a>(tag: Tag, events: Vec<Event<'a>>) -> Vec<Event<'a>> {
    vec![Event::Start(tag)] + events + vec![Event::End(tag.clone())]
}

impl IntoIterator for Inline {
    type Item = Event<'static>;
    type IntoIter = Vec<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Inline::Text(s) => vec![Event::Text(CowStr::Boxed(s.into_boxed_str()))],
            Inline::Emphasis(inner) => wrap_events(Tag::Emphasis, inner.into_iter().collect()),
            Inline::Strong(inner) => wrap_events(Tag::Strong, inner.into_iter().collect()),
            Inline::Strikethrough(inner) => wrap_events(Tag::Strikethrough, inner.into_iter().collect()),
            Inline::Code(s) => vec![Event::Code(CowStr::Boxed(s.into_boxed_str()))],
            Inline::ListItem(inner) => wrap_events(Tag::Item, inner.into_iter().collect()),
            Inline::SoftBreak => vec![Event::SoftBreak],
            Inline::HardBreak => vec![Event::HardBreak],
            Inline::Rule => vec![Event::Rule],
        }
    }
}

impl IntoIterator for Block {
    type Item = Event<'static>;
    type IntoIter = Vec<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Block::Heading { lvl, id, classes, inner } => wrap_events(Tag::Heading(lvl, id.map(|s| s.as_str()), classes.map(|s| s.as_str()).collect()), inner.into_iter().collect()),
            Block::Paragraph(inner) => wrap_events(Tag::Paragraph, inner.into_iter().collect()),
            Block::BlockQuote(inner) => wrap_events(Tag::BlockQuote, inner.into_iter().collect()),
            Block::CodeBlock { .. } => vec![Event::Html(CowStr::from("<strong>CODE</strong>"))],
            Block::List(_, _) => vec![Event::Html(CowStr::from("<strong>LIST</strong>"))],
            Block::Html(s) => vec![Event::Html(CowStr::Boxed(s.into_boxed_str()))],
            Block::Image(tp, url, title) => wrap_events(Tag::Image(tp, CowStr::Boxed(url.into_boxed_str()), CowStr::Boxed(title.into_boxed_str())), Vec::new()),
        }
    }
}

impl<'a> FromIterator<Event<'a>> for Document {
    fn from_iter<T: IntoIterator<Item=Event<'a>>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        let mut blocks = Vec::new();

        let mut inners = Vec::new();

        while let Some(event) = iter.next() {
            match event {
                Event::Start(_) => {
                    // tags.push(t);
                    inners.push(Vec::new());
                }
                Event::End(t) => {
                    let inner = inners.pop().expect("No inner content");
                    match t {
                        Tag::Paragraph => Block::Paragraph(inner),
                        Tag::Heading(lvl, id, classes) => blocks.push(Block::Heading {
                            lvl,
                            id: id.map(|s| s.to_string()),
                            classes: classes.into_iter().map(|s| s.to_string()).collect(),
                            inner,
                        }),
                        Tag::BlockQuote => blocks.push(Block::BlockQuote(inner)),
                        Tag::CodeBlock(variant) => {
                            let info = match variant {
                                CodeBlockKind::Indented => "".to_string(),
                                CodeBlockKind::Fenced(s) => s.to_string()
                            };
                            blocks.push(Block::CodeBlock {
                                source: inner,
                                reference: None,
                                attr: CodeAttributes {},
                                outputs: vec![],
                            });
                        }
                        Tag::List(idx) => Block::List(idx, inner),
                        Tag::Item => inners.first().unwrap().push(Inline::ListItem(inner)),
                        Tag::Emphasis => inners.first().unwrap().push(Inline::Emphasis(inner)),
                        Tag::Strong => inners.first().unwrap().push(Inline::Strong(inner)),
                        Tag::Strikethrough => inners.first().unwrap().push(Inline::Strikethrough(inner)),
                        // Tag::Link(_, _, _) => {}
                        Tag::Image(tp, url, title) => blocks.push(Block::Image(tp, url.to_string(), title.to_string())),
                        _ => unreachable!(),
                    }
                }
                Event::Html(s) => {
                    blocks.push(Block::Html(s.into_string()));
                }
                other => {
                    let inner = match other {
                        Event::Text(s) => Inline::Text(s.into_string()),
                        Event::Code(s) => Inline::Code(s.into_string()),
                        Event::SoftBreak => Inline::SoftBreak,
                        Event::HardBreak => Inline::HardBreak,
                        Event::Rule => Inline::Rule,
                        _ => unreachable!()
                    };

                    inners.first().map(|mut events| events.push(inner))
                }
            }
        }

        Document(blocks)
    }
}