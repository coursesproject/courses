use crate::notebook::*;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, LinkType, Tag};
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
    list_order_num: Option<u64>,
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
            list_order_num: None,
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
            Tag::List(i) => {
                self.list_order_num = i;
            }
            Tag::Item => {
                match self.list_order_num {
                    None => self.cell_source.push_str("- "),
                    Some(i) => {
                        self.cell_source.push_str(&format!("{}. ", i));
                        self.list_order_num = self.list_order_num.map(|i| i+1);
                    }
                }
            }
            Tag::FootnoteDefinition(_) => {}
            Tag::Table(_) => {}
            Tag::TableHead => {}
            Tag::TableRow => {}
            Tag::TableCell => {}
            Tag::Emphasis => self.cell_source.push_str("*"),
            Tag::Strong => self.cell_source.push_str("__"),
            Tag::Strikethrough => {}
            Tag::Link(_, _, _) => self.cell_source.push_str("["),
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
            Tag::Paragraph => self.cell_source.push_str("\n"),
            Tag::Heading(_, _, _) => self.cell_source.push_str("\n\n"),
            Tag::BlockQuote => {}
            Tag::List(i) => self.cell_source.push_str("\n"),
            Tag::Item => self.cell_source.push_str("\n"),
            Tag::FootnoteDefinition(_) => {}
            Tag::Table(_) => {}
            Tag::TableHead => {}
            Tag::TableRow => {}
            Tag::TableCell => {}
            Tag::Emphasis => self.cell_source.push_str("*"),
            Tag::Strong => self.cell_source.push_str("__"),
            Tag::Strikethrough => {}
            Tag::Link(type_, dest, title) => {
                self.cell_source.push_str(format!("]({} {})", dest, title).as_str());
            }
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
                Event::SoftBreak => self.cell_source.push_str("\n"),
                Event::HardBreak => self.cell_source.push_str("\n\n"),
                Event::Rule => {}
                Event::TaskListMarker(_) => {}
            };
        }
        self.finished_cells.push(self.cell_type.to_notebook_format(self.cell_source.clone()));
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


struct MarkdownWriter<I> {
    iter: I,
    source: String,
    list_order_num: Option<u64>,
}

impl<'a, I> MarkdownWriter<I>
    where
        I: Iterator<Item = Event<'a>>,
{
    fn new(iter: I) -> Self {
        MarkdownWriter {
            iter,
            source: String::new(),
            list_order_num: None,
        }
    }

    fn start_tag(&mut self, tag: Tag<'a>) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {}
            Tag::Heading(level, _, _) => {
                let mut prefix = "#".repeat(heading_num(level));
                prefix.push(' ');
                self.source.push_str(&prefix);
            }
            Tag::BlockQuote => {}
            Tag::CodeBlock(kind) => match kind {
                CodeBlockKind::Indented => {
                    self.source.push_str("```plain\n");
                }
                CodeBlockKind::Fenced(cls) => {
                    let s = cls.into_string();
                    self.source.push_str(format!("```{}\n", s).as_str());
                }
            },
            Tag::List(i) => {
                self.list_order_num = i;
            }
            Tag::Item => {
                match self.list_order_num {
                    None => self.source.push_str("- "),
                    Some(i) => {
                        self.source.push_str(&format!("{}. ", i));
                        self.list_order_num = self.list_order_num.map(|i| i+1);
                    }
                }
            }
            Tag::FootnoteDefinition(_) => {}
            Tag::Table(_) => {}
            Tag::TableHead => {}
            Tag::TableRow => {}
            Tag::TableCell => {}
            Tag::Emphasis => self.source.push_str("*"),
            Tag::Strong => self.source.push_str("__"),
            Tag::Strikethrough => {}
            Tag::Link(_, _, _) => self.source.push_str("["),
            Tag::Image(_, _, _) => {}
        }
        Ok(())
    }

    fn end_tag(&mut self, tag: Tag<'a>) -> io::Result<()> {
        match tag {
            Tag::CodeBlock(kind) => self.source.push_str("\n```\n"),
            Tag::Paragraph => self.source.push_str("\n"),
            Tag::Heading(_, _, _) => self.source.push_str("\n\n"),
            Tag::BlockQuote => {}
            Tag::List(i) => self.source.push_str("\n"),
            Tag::Item => self.source.push_str("\n"),
            Tag::FootnoteDefinition(_) => {}
            Tag::Table(_) => {}
            Tag::TableHead => {}
            Tag::TableRow => {}
            Tag::TableCell => {}
            Tag::Emphasis => self.source.push_str("*"),
            Tag::Strong => self.source.push_str("__"),
            Tag::Strikethrough => {}
            Tag::Link(type_, dest, title) => {
                self.source.push_str(format!("]({} {})", dest, title).as_str());
            }
            Tag::Image(_, _, _) => {}
        }
        Ok(())
    }

    fn run(mut self) -> io::Result<String> {
        while let Some(event) = self.iter.next() {
            match event {
                Event::Start(tag) => self.start_tag(tag)?,
                Event::End(tag) => self.end_tag(tag)?,
                Event::Text(text) => self.source.push_str(&text.into_string()),
                Event::Code(_) => {}
                Event::Html(text) => self.source.push_str(&text.into_string()),
                Event::FootnoteReference(_) => {}
                Event::SoftBreak => self.source.push_str("\n"),
                Event::HardBreak => self.source.push_str("\n\n"),
                Event::Rule => {}
                Event::TaskListMarker(_) => {}
            };
        }

        Ok(self.source)
    }
}

pub fn render_markdown<'a, I>(iter: I) -> io::Result<String>
    where
        I: Iterator<Item = Event<'a>>,
{
    MarkdownWriter::new(iter).run()
}