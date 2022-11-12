use std::fmt::{Display, Formatter};
use std::ops::Range;
use crate::extensions::shortcode_extender::{ShortCodeProcessError, ShortCodeProcessor};
use crate::notebook::{Cell, CellEventIterator, CellOutput, Notebook};
use pulldown_cmark::CodeBlockKind::Fenced;
use pulldown_cmark::Tag::CodeBlock;
use pulldown_cmark::{CowStr, Event, OffsetIter, Options, Parser};
use std::vec::IntoIter;
use thiserror::Error;

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
    global_offset: usize,
    line: usize,
    local_position: Range<usize>
}

impl Display for DocPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.cell_number {
            None => write!(f, "line: {}", self.line),
            Some(n) => write!(f, "cell: {}, local position: {}", n, self.line)
        }

    }
}

impl DocPos {
    pub fn new(cell_number: Option<usize>, global_offset: usize, line: usize, local_position: Range<usize>) -> Self {
        DocPos {
            cell_number, global_offset, line, local_position
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Document {
    elements: Vec<Element>,
}

#[derive(Error, Debug)]
pub enum PreprocessError {
    #[error(transparent)]
    Shortcode(#[from] ShortCodeProcessError),
}

impl Document {
    pub fn preprocess(&self, processor: &ShortCodeProcessor) -> Result<Document, PreprocessError> {
        let elements = self
            .elements
            .iter()
            .map(|e| match e {
                Element::Markdown { content } => Ok(Element::Markdown {
                    content: processor.process(content)?,
                }),
                _ => Ok(e.clone()),
            })
            .collect::<Result<Vec<Element>, ShortCodeProcessError>>()?;
        Ok(Document { elements })
    }
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
            .fold((1, Vec::new()) , |(num, mut acc), cell| {
                let next = match &cell {
                    Cell::Code { .. } => num+1,
                    _ => num
                };
                acc.push((next, cell));
                (next, acc)
            }).1.into_iter()
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

        Document { elements }
    }
}

pub struct ElementIterator<'a, 'b> {
    global_offset: usize,
    source: String,
    cell_iter: ElementIteratorCell<'a, 'b>
}

pub enum ElementIteratorCell<'a, 'b> {
    Markdown { parser: OffsetIter<'a, 'b> },
    Code { cell_number: usize, events: IntoIter<(Event<'a>, Range<usize>)> },
    Raw {},
}

impl<'a, 'b> ElementIterator<'a, 'b> {
    fn map_doc_pos(&self, elem: (Event<'a>, Range<usize>)) -> (Event<'a>, DocPos) {
        let cell_num = match &self.cell_iter {
            ElementIteratorCell::Code { cell_number, .. } => Some(*cell_number),
            _ => None,
        };
        let line = &self.source[elem.1.start .. elem.1.end].lines().count();

        (elem.0, DocPos::new(cell_num, self.global_offset, *line, elem.1))
    }
}

impl<'a, 'b> Iterator for ElementIterator<'a, 'b> {
    type Item = (Event<'a>, DocPos);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.cell_iter {
            ElementIteratorCell::Markdown { parser, .. } => parser.next().map(|e| self.map_doc_pos(e)),
            ElementIteratorCell::Code { events, .. } => events.next().map(|e| self.map_doc_pos(e)),
            ElementIteratorCell::Raw { .. } => None,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct IteratorConfig {
    include_output: bool,
    include_solutions: bool,
}

pub struct IteratorConfigBuilder {
    include_output: Option<bool>,
    include_solutions: Option<bool>,
}

impl IteratorConfig {
    pub fn include_output(self) -> Self {
        IteratorConfig {
            include_output: true,
            include_solutions: self.include_solutions,
        }
    }
    pub fn include_solutions(self) -> Self {
        IteratorConfig {
            include_output: self.include_output,
            include_solutions: true,
        }
    }
}

pub trait ConfigureIterator {
    type Item;
    type IntoIter;

    fn configure_iterator(self, config: IteratorConfig) -> Self::IntoIter;
}

pub trait ConfigureElemIterator {
    type Item;
    type IntoIter;

    fn configure_iterator(self, cell_number: usize, config: IteratorConfig) -> Self::IntoIter;
}

impl<'a> ConfigureIterator for &'a Element {
    type Item = Event<'a>;
    type IntoIter = ElementIterator<'a, 'a>;

    fn configure_iterator(self, config: IteratorConfig) -> Self::IntoIter {
        let (cell, content) = match self {
            Element::Markdown { content } => (ElementIteratorCell::Markdown {
                parser: Parser::new_ext(&content, Options::all()).into_offset_iter(),
            }, content.clone()),

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

                (ElementIteratorCell::Code {
                    cell_number: *cell_number,
                    events: events.into_iter(),
                }, content.clone())
            }
            Element::Raw { content } => (ElementIteratorCell::Raw {}, content.clone()),
            _ => (ElementIteratorCell::Raw {}, "".to_string()),
        };
        ElementIterator {
            source: content,
            global_offset: 0,
            cell_iter: cell
        }
    }
}

impl<'a> ConfigureIterator for &'a Document {
    type Item = (Event<'a>, DocPos);
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn configure_iterator(self, config: IteratorConfig) -> Self::IntoIter {
        Box::new(
            self.elements
                .iter()
                .flat_map(move |elem: &Element| elem.configure_iterator( config)),
        )
    }
}
