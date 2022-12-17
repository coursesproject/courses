use crate::ast::AEvent;
use crate::notebook::{Cell, CellOutput, Notebook};
use crate::processors::shortcode_extender::ShortCodeProcessError;
use crate::processors::Preprocessor;
use crate::Context;
use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::Tag::CodeBlock;
use pulldown_cmark::{CowStr, Event, OffsetIter, Options, Parser};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::rc::Rc;
use std::vec::IntoIter;
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct RawDocument {
    pub metadata: DocumentMetadata,
    content: Content,
}

#[derive(Debug, Clone, Default)]
pub struct EventDocument {
    pub metadata: DocumentMetadata,
    pub content: Vec<(AEvent, DocPos)>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Content(Vec<Element>);

#[derive(Debug, Clone, Default)]
pub enum Element {
    Markdown {
        content: String,
    },
    Code {
        cell_number: usize,
        content: String,
        output: Option<Vec<CellOutput>>,
    },
    Raw {
        content: String,
    },
    #[default]
    Default,
}

#[derive(Debug, Clone)]
pub struct DocPos {
    cell_number: Option<usize>,
    #[allow(unused)]
    global_offset: usize,
    line: usize,
    #[allow(unused)]
    local_position: Range<usize>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    #[serde(rename = "type", default = "default_title")]
    pub doc_type: String,
    #[serde(default = "default_true")]
    pub code_split: bool,
    #[serde(default = "default_true")]
    pub notebook_output: bool,
    #[serde(default)]
    pub layout: LayoutSettings,

    #[serde(default)]
    pub output: OutputSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputSpec {
    #[serde(default = "default_true")]
    pub web: bool,
    #[serde(default = "default_true")]
    pub source: bool,
}

impl Default for OutputSpec {
    fn default() -> Self {
        OutputSpec {
            web: true,
            source: true,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LayoutSettings {
    pub hide_sidebar: bool,
}

fn default_title() -> String {
    "text".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Error, Debug)]
pub enum PreprocessError {
    #[error(transparent)]
    Shortcode(#[from] ShortCodeProcessError),
}

impl Display for DocPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.cell_number {
            None => write!(f, "line: {}", self.line),
            Some(n) => write!(f, "cell: {}, local position: {}", n, self.line),
        }
    }
}

impl DocPos {
    pub fn new(
        cell_number: Option<usize>,
        global_offset: usize,
        line: usize,
        local_position: Range<usize>,
    ) -> Self {
        DocPos {
            cell_number,
            global_offset,
            line,
            local_position,
        }
    }
}

impl RawDocument {
    pub fn preprocess(
        self,
        processor: &dyn Preprocessor,
        ctx: &Context,
    ) -> Result<RawDocument, Box<dyn std::error::Error>> {
        let elements = self
            .content
            .0
            .iter()
            .map(|e| match e {
                Element::Markdown { content } => Ok(Element::Markdown {
                    content: processor.process(content, ctx)?,
                }),
                _ => Ok(e.clone()),
            })
            .collect::<Result<Vec<Element>, Box<dyn std::error::Error>>>()?;
        Ok(RawDocument {
            content: Content(elements),
            metadata: self.metadata,
        })
    }

    pub(crate) fn new<C: Into<Content>>(content: C, metadata: DocumentMetadata) -> Self {
        RawDocument {
            content: content.into(),
            metadata,
        }
    }

    pub fn to_events(&self, config: IteratorConfig) -> EventDocument {
        let content = self.configure_iterator(config).map(|(e, p)| (e.into(), p));
        EventDocument {
            metadata: self.metadata.clone(),
            content: content.collect(),
        }
    }
}

impl EventDocument {
    pub fn to_events(&self) -> impl Iterator<Item = Event<'static>> {
        self.content.clone().into_iter().map(|(e, p)| e.into())
    }

    pub fn to_events_with_pos(&self) -> impl Iterator<Item = (Event<'static>, DocPos)> {
        self.content.clone().into_iter().map(|(e, p)| (e.into(), p))
    }
}

impl From<String> for Content {
    fn from(s: String) -> Self {
        Content(vec![Element::Markdown { content: s }])
    }
}

impl From<Notebook> for Content {
    fn from(n: Notebook) -> Self {
        let elements = n
            .cells
            .into_iter()
            .fold((1, Vec::new()), |(num, mut acc), cell| {
                let next = match &cell {
                    Cell::Code { .. } => num + 1,
                    _ => num,
                };
                acc.push((next, cell));
                (next, acc)
            })
            .1
            .into_iter()
            .map(|(i, cell)| match cell {
                Cell::Markdown { common } => Element::Markdown {
                    content: common.source,
                },
                Cell::Code {
                    common, outputs, ..
                } => Element::Code {
                    cell_number: i,
                    content: common.source,
                    output: Some(outputs),
                },
                Cell::Raw { common } => Element::Raw {
                    content: common.source,
                },
            })
            .collect();

        Content(elements)
    }
}

pub struct ElementIterator<'a, 'b> {
    global_offset: usize,
    source: String,
    cell_iter: ElementIteratorCell<'a, 'b>,
}

pub enum ElementIteratorCell<'a, 'b> {
    Markdown {
        parser: Box<OffsetIter<'a, 'b>>,
    },
    Code {
        cell_number: usize,
        events: Box<IntoIter<(Event<'a>, Range<usize>)>>,
    },
    Raw {},
}

impl<'a, 'b> ElementIterator<'a, 'b> {
    fn map_doc_pos(&self, elem: (Event<'a>, Range<usize>)) -> (Event<'a>, DocPos) {
        let cell_num = match &self.cell_iter {
            ElementIteratorCell::Code { cell_number, .. } => Some(*cell_number),
            _ => None,
        };
        let line = &self.source[elem.1.start..elem.1.end].lines().count();

        (
            elem.0,
            DocPos::new(cell_num, self.global_offset, *line, elem.1),
        )
    }
}

impl<'a, 'b> Iterator for ElementIterator<'a, 'b> {
    type Item = (Event<'a>, DocPos);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.cell_iter {
            ElementIteratorCell::Markdown { parser, .. } => {
                parser.next().map(|e| self.map_doc_pos(e))
            }
            ElementIteratorCell::Code { events, .. } => events.next().map(|e| self.map_doc_pos(e)),
            ElementIteratorCell::Raw { .. } => None,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct IteratorConfig {
    pub include_output: bool,
    pub include_solutions: bool,
}

impl IteratorConfig {
    #[allow(unused)]
    pub fn include_output(self) -> Self {
        IteratorConfig {
            include_output: true,
            include_solutions: self.include_solutions,
        }
    }

    #[allow(unused)]
    pub fn include_solutions(self) -> Self {
        IteratorConfig {
            include_output: self.include_output,
            include_solutions: true,
        }
    }
}

pub trait ConfigureCollector {
    type Item;
    type IntoIter;

    fn configure_iterator(self, config: IteratorConfig) -> Self::IntoIter;
}

pub trait ConfigureElemIterator {
    type Item;
    type IntoIter;

    fn configure_iterator(self, cell_number: usize, config: IteratorConfig) -> Self::IntoIter;
}

impl<'a> ConfigureCollector for &'a Element {
    type Item = Event<'a>;
    type IntoIter = ElementIterator<'a, 'a>;

    fn configure_iterator(self, config: IteratorConfig) -> Self::IntoIter {
        let (cell, content) = match self {
            Element::Markdown { content } => (
                ElementIteratorCell::Markdown {
                    parser: Box::new(Parser::new_ext(content, Options::all()).into_offset_iter()),
                },
                content.clone(),
            ),

            Element::Code {
                cell_number,
                content,
                output: outputs,
            } => {
                let cblock = CodeBlock(Fenced(CowStr::Boxed("python".into())));
                let mut events = vec![
                    (Event::Start(cblock.clone()), (0..0)),
                    (Event::Text(CowStr::Borrowed(content)), (0..content.len())),
                    (Event::End(cblock), (content.len()..content.len())),
                ];
                if config.include_output {
                    if let Some(os) = outputs {
                        for o in os {
                            events.append(&mut o.to_events());
                        }
                    }
                }

                (
                    ElementIteratorCell::Code {
                        cell_number: *cell_number,
                        events: Box::new(events.into_iter()),
                    },
                    content.clone(),
                )
            }
            Element::Raw { content } => (ElementIteratorCell::Raw {}, content.clone()),
            _ => (ElementIteratorCell::Raw {}, "".to_string()),
        };
        ElementIterator {
            source: content,
            global_offset: 0,
            cell_iter: cell,
        }
    }
}

impl<'a> ConfigureCollector for &'a RawDocument {
    type Item = (Event<'a>, DocPos);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn configure_iterator(self, config: IteratorConfig) -> Self::IntoIter {
        Box::new(
            self.content
                .0
                .iter()
                .flat_map(move |elem: &Element| elem.configure_iterator(config)),
        )
    }
}
