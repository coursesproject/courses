use crate::ast::{Ast, Block};
use anyhow::Result;
use pulldown_cmark::HeadingLevel;
use serde::{Deserialize, Serialize};

use crate::document::Document;
use crate::notebook::{Cell, CellCommon, Notebook, NotebookMeta};
use crate::renderers::markdown::{ToMarkdown, ToMarkdownContext};
use crate::renderers::{RenderContext, RenderResult, Renderer};

#[derive(Serialize, Deserialize)]
pub struct NotebookRenderer;

#[typetag::serde(name = "renderer_config")]
impl Renderer for NotebookRenderer {
    fn render(&self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<Document<RenderResult>> {
        let markdown_ctx = ToMarkdownContext {
            metadata: doc.metadata.clone(),
            ids: doc.ids.clone(),
            ids_map: doc.id_map.clone(),
            templates: ctx.templates.clone(),
            extra_args: ctx.extra_args.clone(),
            list_idx: None,
            list_lvl: 0,
        };

        let writer = NotebookWriter {
            cell_source: String::new(),
            finished_cells: vec![],
            markdown_ctx,
            notebook_meta: ctx.notebook_output_meta.clone(),
        };

        let notebook: Notebook = writer.convert(doc.content.clone())?;
        let output = serde_json::to_string(&notebook).expect("Invalid notebook (this is a bug)");

        Ok(Document {
            content: output,
            metadata: doc.metadata.clone(),
            variables: doc.variables.clone(),
            ids: doc.ids.clone(),
            id_map: doc.id_map.clone(),
        })
    }
}

pub fn heading_num(h: HeadingLevel) -> usize {
    match h {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

struct NotebookWriter {
    cell_source: String,
    finished_cells: Vec<Cell>,
    notebook_meta: NotebookMeta,
    markdown_ctx: ToMarkdownContext,
}

impl NotebookWriter {
    fn push_markdown_cell(&mut self) {
        self.finished_cells.push(Cell::Markdown {
            common: CellCommon {
                metadata: Default::default(),
                source: self.cell_source.clone(),
            },
        });
        self.cell_source = String::new();
    }

    fn block(&mut self, block: Block) -> Result<()> {
        match block {
            Block::CodeBlock { source, .. } => {
                self.push_markdown_cell();
                let c = Cell::Code {
                    common: CellCommon {
                        metadata: Default::default(),
                        source,
                    },
                    execution_count: None,
                    outputs: Vec::new(),
                };
                self.finished_cells.push(c);
            }
            _ => self
                .cell_source
                .push_str(&block.to_markdown(&mut self.markdown_ctx)?),
        }

        Ok(())
    }

    fn convert(mut self, ast: Ast) -> Result<Notebook> {
        for b in ast.0 {
            self.block(b)?;
        }

        Ok(Notebook {
            metadata: self.notebook_meta,
            nbformat: 4,
            nbformat_minor: 5,
            cells: self.finished_cells,
        })
    }
}
//
// struct NotebookWriterOld<I> {
//     iter: I,
//     cell_type: CellType,
//     cell_source: String,
//     finished_cells: Vec<Cell>,
//     list_order_num: Option<u64>,
// }
//
// impl<'a, I> NotebookWriterOld<I>
// where
//     I: Iterator<Item = Event<'a>>,
// {
//     fn new(iter: I) -> Self {
//         NotebookWriterOld {
//             iter,
//             cell_type: CellType::Markdown,
//             cell_source: String::new(),
//             finished_cells: Vec::new(),
//             list_order_num: None,
//         }
//     }
//
//     fn start_tag(&mut self, tag: Tag<'a>) {
//         match tag {
//             Tag::Paragraph => {}
//             Tag::Heading(level, _, _) => {
//                 let mut prefix = "#".repeat(heading_num(level));
//                 prefix.push(' ');
//                 self.cell_source.push_str(&prefix);
//             }
//             Tag::BlockQuote => {}
//             Tag::CodeBlock(kind) => match kind {
//                 CodeBlockKind::Indented => {
//                     self.cell_source.push_str("```plain\n");
//                 }
//                 CodeBlockKind::Fenced(cls) => {
//                     let s = cls.into_string();
//                     match s.as_str() {
//                         "python" => {
//                             self.finished_cells
//                                 .push(self.cell_type.to_notebook_format(self.cell_source.clone()));
//                             self.cell_source = String::new();
//                             self.cell_type = CellType::Code;
//                         }
//                         _ => {
//                             self.cell_source.push_str("```plain\n");
//                         }
//                     }
//                 }
//             },
//             Tag::List(i) => {
//                 self.list_order_num = i;
//             }
//             Tag::Item => match self.list_order_num {
//                 None => self.cell_source.push_str("- "),
//                 Some(i) => {
//                     write!(self.cell_source, "{}. ", i).expect("Invalid format");
//                     self.list_order_num = self.list_order_num.map(|i| i + 1);
//                 }
//             },
//             Tag::FootnoteDefinition(_) => {}
//             Tag::Table(_) => {}
//             Tag::TableHead => {}
//             Tag::TableRow => {}
//             Tag::TableCell => {}
//             Tag::Emphasis => self.cell_source.push('*'),
//             Tag::Strong => self.cell_source.push_str("__"),
//             Tag::Strikethrough => {}
//             Tag::Link(_, _, _) => self.cell_source.push('['),
//             Tag::Image(_, _, _) => {}
//         }
//     }
//
//     fn end_tag(&mut self, tag: Tag<'a>) {
//         match tag {
//             Tag::CodeBlock(kind) => match kind {
//                 CodeBlockKind::Indented => {
//                     self.cell_source.push_str("\n```\ngit pull");
//                 }
//                 CodeBlockKind::Fenced(cls) => {
//                     let s = cls.into_string();
//                     match s.as_str() {
//                         "python" => {
//                             self.finished_cells
//                                 .push(self.cell_type.to_notebook_format(self.cell_source.clone()));
//                             self.cell_source = String::new();
//                             self.cell_type = CellType::Markdown;
//                         }
//                         _ => {
//                             self.cell_source.push_str("\n```\n");
//                         }
//                     }
//                 }
//             },
//             Tag::Paragraph => self.cell_source.push('\n'),
//             Tag::Heading(_, _, _) => self.cell_source.push_str("\n\n"),
//             Tag::BlockQuote => {}
//             Tag::List(_) => self.cell_source.push('\n'),
//             Tag::Item => self.cell_source.push('\n'),
//             Tag::FootnoteDefinition(_) => {}
//             Tag::Table(_) => {}
//             Tag::TableHead => {}
//             Tag::TableRow => {}
//             Tag::TableCell => {}
//             Tag::Emphasis => self.cell_source.push('*'),
//             Tag::Strong => self.cell_source.push_str("__"),
//             Tag::Strikethrough => {}
//             Tag::Link(_type, dest, title) => {
//                 self.cell_source
//                     .push_str(format!("]({} {})", dest, title).as_str());
//             }
//             Tag::Image(_, _, _) => {}
//         }
//     }
//
//     fn run(mut self) -> Notebook {
//         while let Some(event) = self.iter.next() {
//             match event {
//                 Event::Start(tag) => self.start_tag(tag),
//                 Event::End(tag) => self.end_tag(tag),
//                 Event::Text(text) => {
//                     let ts = text.into_string();
//                     if &ts == "\\" {
//                         self.cell_source.push_str("\\\\");
//                     } else {
//                         self.cell_source.push_str(&ts)
//                     }
//                 }
//                 Event::Code(_) => {}
//                 Event::Html(text) => self.cell_source.push_str(&text.into_string()),
//                 Event::FootnoteReference(_) => {}
//                 Event::SoftBreak => self.cell_source.push('\n'),
//                 Event::HardBreak => self.cell_source.push_str("\n\n"),
//                 Event::Rule => {}
//                 Event::TaskListMarker(_) => {}
//             };
//         }
//         self.finished_cells
//             .push(self.cell_type.to_notebook_format(self.cell_source.clone()));
//         Notebook {
//             metadata: NotebookMeta {
//                 kernelspec: None,
//                 optional: HashMap::new(),
//             },
//             nbformat: 4,
//             nbformat_minor: 4,
//             cells: self.finished_cells,
//         }
//     }
// }
//
// pub fn render_notebook<'a, I>(iter: I) -> Notebook
// where
//     I: Iterator<Item = Event<'a>>,
// {
//     NotebookWriterOld::new(iter).run()
// }
