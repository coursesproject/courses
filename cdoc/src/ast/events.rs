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

impl<'a> From<&'a ATag> for Tag<'a> {
    fn from(t: &'a ATag) -> Self {
        match t {
            ATag::Paragraph => Tag::Paragraph,
            ATag::Heading(lvl, id, _) => {
                Tag::Heading(*lvl, id.as_ref().map(|s| s.as_str()), vec![])
            }
            ATag::BlockQuote => Tag::BlockQuote,
            ATag::CodeBlock(c) => Tag::CodeBlock(c.clone().into()),
            ATag::List(n) => Tag::List(*n),
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
                *typ,
                CowStr::Boxed(url.clone().into_boxed_str()),
                CowStr::Boxed(alt.clone().into_boxed_str()),
            ),
            ATag::Image(typ, url, alt) => Tag::Image(
                *typ,
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
