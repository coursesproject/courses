use crate::notebook::*;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Tag};
use serde::de::Unexpected::Str;
use std::collections::HashMap;
use std::io;

enum CellType {
    Markdown,
    Code,
    Raw,
}

impl CellType {
    fn to_notebook_format(&self, source: String) -> Cell {
        let common = CellCommon {
            source,
            metadata: CellMeta::default(),
        };
        match self {
            CellType::Markdown => Cell::Markdown { common },
            CellType::Code => Cell::Code {
                common,
                execution_count: None,
                outputs: Vec::new(),
            },
            CellType::Raw => Cell::Raw { common },
        }
    }
}

fn heading_num(h: HeadingLevel) -> usize {
    match h {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

struct NotebookWriter<I> {
    iter: I,
    cell_type: CellType,
    cell_source: String,
    finished_cells: Vec<Cell>,
}

impl<'a, I> NotebookWriter<I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn new(iter: I) -> Self {
        NotebookWriter {
            iter,
            cell_type: CellType::Markdown,
            cell_source: String::new(),
            finished_cells: Vec::new(),
        }
    }

    fn start_tag(&mut self, tag: Tag<'a>) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {}
            Tag::Heading(level, _, _) => {
                let mut prefix = "#".repeat(heading_num(level));
                prefix.push(' ');
                self.cell_source.push_str(&prefix);
            }
            Tag::BlockQuote => {}
            Tag::CodeBlock(kind) => match kind {
                CodeBlockKind::Indented => {
                    self.cell_source.push_str("```plain\n");
                }
                CodeBlockKind::Fenced(cls) => {
                    let s = cls.into_string();
                    match s.as_str() {
                        "python" => {
                            self.finished_cells
                                .push(self.cell_type.to_notebook_format(self.cell_source.clone()));
                            self.cell_source = String::new();
                            self.cell_type = CellType::Code;
                        }
                        _ => {
                            self.cell_source.push_str("```plain\n");
                        }
                    }
                }
            },
            Tag::List(_) => {}
            Tag::Item => {}
            Tag::FootnoteDefinition(_) => {}
            Tag::Table(_) => {}
            Tag::TableHead => {}
            Tag::TableRow => {}
            Tag::TableCell => {}
            Tag::Emphasis => {}
            Tag::Strong => {}
            Tag::Strikethrough => {}
            Tag::Link(_, _, _) => {}
            Tag::Image(_, _, _) => {}
        }
        Ok(())
    }

    fn end_tag(&mut self, tag: Tag<'a>) -> io::Result<()> {
        match tag {
            Tag::CodeBlock(kind) => match kind {
                CodeBlockKind::Indented => {
                    self.cell_source.push_str("\n```\ngit pull");
                }
                CodeBlockKind::Fenced(cls) => {
                    let s = cls.into_string();
                    match s.as_str() {
                        "python" => {
                            self.finished_cells
                                .push(self.cell_type.to_notebook_format(self.cell_source.clone()));
                            self.cell_source = String::new();
                            self.cell_type = CellType::Markdown;
                        }
                        _ => {
                            self.cell_source.push_str("\n```\n");
                        }
                    }
                }
            },
            Tag::Paragraph => self.cell_source.push_str("\n "),
            Tag::Heading(_, _, _) => self.cell_source.push_str("\n "),
            Tag::BlockQuote => {}
            Tag::List(_) => {}
            Tag::Item => {}
            Tag::FootnoteDefinition(_) => {}
            Tag::Table(_) => {}
            Tag::TableHead => {}
            Tag::TableRow => {}
            Tag::TableCell => {}
            Tag::Emphasis => {}
            Tag::Strong => {}
            Tag::Strikethrough => {}
            Tag::Link(_, _, _) => {}
            Tag::Image(_, _, _) => {}
        }
        Ok(())
    }

    fn run(mut self) -> io::Result<Notebook> {
        while let Some(event) = self.iter.next() {
            match event {
                Event::Start(tag) => self.start_tag(tag)?,
                Event::End(tag) => self.end_tag(tag)?,
                Event::Text(text) => self.cell_source.push_str(&text.into_string()),
                Event::Code(_) => {}
                Event::Html(text) => self.cell_source.push_str(&text.into_string()),
                Event::FootnoteReference(_) => {}
                Event::SoftBreak => {}
                Event::HardBreak => self.cell_source.push_str("\\n"),
                Event::Rule => {}
                Event::TaskListMarker(_) => {}
            };
        }
        Ok(Notebook {
            metadata: NotebookMeta {
                kernelspec: None,
                optional: HashMap::new(),
            },
            nbformat: 4,
            nbformat_minor: 4,
            cells: self.finished_cells,
        })
    }
}

pub fn render_notebook<'a, I>(iter: I) -> io::Result<Notebook>
where
    I: Iterator<Item = Event<'a>>,
{
    NotebookWriter::new(iter).run()
}
