use crate::notebook::{Cell, Notebook};
use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::Tag::CodeBlock;
use pulldown_cmark::{CowStr, Event, Options, Parser};
use std::iter::FlatMap;
use std::slice::Iter;
use std::vec::IntoIter;

pub struct CellOutput {}

pub enum Element {
    Markdown {
        content: String,
    },
    Code {
        content: String,
        output: Option<CellOutput>,
    },
    Raw {
        content: String,
    },
}

pub struct Document {
    elements: Vec<Element>,
}

impl From<String> for Document {
    fn from(s: String) -> Self {
        Document {
            elements: vec![Element::Markdown { content: s }],
        }
    }
}

impl From<Notebook> for Document {
    fn from(n: Notebook) -> Self {
        let elements = n
            .cells
            .into_iter()
            .map(|cell| match cell {
                Cell::Markdown { common } => Element::Markdown {
                    content: common.source,
                },
                Cell::Code {
                    common, outputs, ..
                } => Element::Code {
                    content: common.source,
                    output: None,
                },
                Cell::Raw { common } => Element::Raw {
                    content: common.source,
                },
            })
            .collect();

        Document { elements }
    }
}

pub enum ElementIterator<'a, 'b> {
    Markdown { parser: Parser<'a, 'b> },
    Code { events: IntoIter<Event<'a>> },
    Raw {},
}

impl<'a, 'b> Iterator for ElementIterator<'a, 'b> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<'a> IntoIterator for &'a Element {
    type Item = Event<'a>;
    type IntoIter = ElementIterator<'a, 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Element::Markdown { content } => ElementIterator::Markdown {
                parser: Parser::new_ext(&content, Options::all()),
            },

            Element::Code { content, .. } => {
                let cblock = CodeBlock(Fenced(CowStr::Boxed("python".into())));
                let mut events = vec![
                    Event::Start(cblock.clone()),
                    Event::Text(CowStr::Borrowed(content)),
                    Event::End(cblock),
                ];
                // outputs
                //     .into_iter()
                //     .for_each(|o| events.append(&mut o.to_events()));
                ElementIterator::Code {
                    events: events.into_iter(),
                }
            }
            Element::Raw { .. } => ElementIterator::Raw {},
        }
    }
}

pub struct DocumentIterator<'a, 'b> {
    iter: FlatMap<
        Iter<'a, Element>,
        ElementIterator<'a, 'b>,
        fn(&'a Element) -> ElementIterator<'a, 'b>,
    >,
}

impl<'a, 'b> Iterator for DocumentIterator<'a, 'b> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, 'b> IntoIterator for &'a Document {
    type Item = Event<'a>;
    type IntoIter = DocumentIterator<'a, 'a>;

    fn into_iter(self) -> Self::IntoIter {
        DocumentIterator {
            iter: self.elements.iter().flat_map(|elem| elem.into_iter()),
        }
    }
}
