use anyhow::Result;
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Ast, Block, Inline};
use cdoc_parser::document::{CodeOutput, Document};
use cdoc_parser::notebook::{Cell, CellCommon, CellMeta, JupyterLabMeta, Notebook, NotebookMeta};
use pulldown_cmark::HeadingLevel;
use replace_with::replace_with_or_abort;
use serde::{Deserialize, Serialize};
use std::io::BufWriter;

use crate::renderers::generic::GenericRenderer;
use crate::renderers::{DocumentRenderer, RenderContext, RenderElement, RenderResult};

pub struct NotebookRendererBuilder;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NotebookRenderer;

#[typetag::serde(name = "notebook")]
impl DocumentRenderer for NotebookRenderer {
    fn render_doc(&mut self, ctx: &RenderContext) -> Result<Document<RenderResult>> {
        let renderer = GenericRenderer::default();

        let writer = NotebookWriter {
            notebook_meta: ctx.notebook_output_meta.clone(),
            outputs: ctx.doc.code_outputs.clone(),
            code_cells: vec![],
            ctx,
            renderer,
        };

        let notebook: Notebook = writer.convert(ctx.doc.content.clone())?;
        let output =
            serde_json::to_string_pretty(&notebook).expect("Invalid notebook (this is a bug)");

        Ok(Document {
            content: output,
            meta: ctx.doc.meta.clone(),
            references: ctx.doc.references.clone(),
            code_outputs: ctx.doc.code_outputs.clone(),
        })
    }
}
//
// pub fn heading_num(h: HeadingLevel) -> usize {
//     match h {
//         HeadingLevel::H1 => 1,
//         HeadingLevel::H2 => 2,
//         HeadingLevel::H3 => 3,
//         HeadingLevel::H4 => 4,
//         HeadingLevel::H5 => 5,
//         HeadingLevel::H6 => 6,
//     }
// }
//
// struct NotebookWriter<'a> {
//     cell_source: Vec<u8>,
//     finished_cells: Vec<Cell>,
//     notebook_meta: NotebookMeta,
//     ctx: &'a RenderContext<'a>,
//     renderer: GenericRenderer,
// }
//
// impl NotebookWriter<'_> {
//     fn push_markdown_cell(&mut self) {
//         replace_with_or_abort(&mut self.cell_source, |source| {
//             if !source.is_empty() {
//                 self.finished_cells.push(Cell::Markdown {
//                     common: CellCommon {
//                         metadata: Default::default(),
//                         source: String::from_utf8(source).unwrap(),
//                     },
//                 });
//             }
//             Vec::new()
//         })
//     }
//
//     fn block(&mut self, block: &Block) -> Result<()> {
//         match block {
//             Block::CodeBlock {
//                 source,
//                 display_cell,
//                 ..
//             } => {
//                 if *display_cell {
//                     self.push_markdown_cell();
//                     let c = Cell::Code {
//                         common: CellCommon {
//                             metadata: Default::default(),
//                             source: source.clone(),
//                         },
//                         execution_count: None,
//                         outputs: Vec::new(),
//                     };
//                     self.finished_cells.push(c);
//                 } else {
//                     self.renderer
//                         .render(block, self.ctx, &mut self.cell_source)?;
//                 }
//             }
//             _ => {
//                 self.renderer
//                     .render(block, self.ctx, &mut self.cell_source)?;
//             }
//         }
//
//         Ok(())
//     }
//
//     fn convert(mut self, ast: Ast) -> Result<Notebook> {
//         let cell_meta = CellMeta {
//             jupyter: Some(JupyterLabMeta {
//                 outputs_hidden: None,
//                 source_hidden: Some(true),
//             }),
//             ..Default::default()
//         };
//         self.finished_cells.push(Cell::Code {
//             common: CellCommon {
//                 metadata: cell_meta,
//                 source: r#"import requests
// from IPython.core.display import HTML
// HTML(f"""
// <style>
// @import "https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css";
// </style>
// """)"#
//                     .to_string(),
//             },
//             execution_count: None,
//             outputs: vec![],
//         });
//
//         for b in &ast.0 {
//             self.block(b)?;
//         }
//
//         self.push_markdown_cell();
//
//         Ok(Notebook {
//             metadata: self.notebook_meta,
//             nbformat: 4,
//             nbformat_minor: 5,
//             cells: self.finished_cells,
//         })
//     }
// }

pub struct NotebookWriter<'a> {
    pub notebook_meta: NotebookMeta,
    pub outputs: Vec<CodeOutput>,
    pub code_cells: Vec<Cell>,
    pub ctx: &'a RenderContext<'a>,
    pub renderer: GenericRenderer,
}

impl NotebookWriter<'_> {
    fn convert(mut self, mut ast: Ast) -> Result<Notebook> {
        let cell_meta = CellMeta {
            jupyter: Some(JupyterLabMeta {
                outputs_hidden: None,
                source_hidden: Some(true),
            }),
            ..Default::default()
        };
        // TODO: Insert this stuff
        //         self.finished_cells.push(Cell::Code {
        //             common: CellCommon {
        //                 metadata: cell_meta,
        //                 source: r#"import requests
        // from IPython.core.display import HTML
        // HTML(f"""
        // <style>
        // @import "https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css";
        // </style>
        // """)"#
        //                     .to_string(),
        //             },
        //             execution_count: None,
        //             outputs: vec![],
        //         });

        self.walk_ast(&mut ast.0)?;

        let mut buf = BufWriter::new(Vec::new());

        self.renderer.render(&ast.0, &self.ctx, &mut buf)?;

        let out_str = String::from_utf8(buf.into_inner()?)?;

        let md_cells = out_str.split(CODE_SPLIT);
        let cells = md_cells
            .zip(self.code_cells)
            .flat_map(|(md, code)| {
                [
                    Cell::Markdown {
                        common: CellCommon {
                            metadata: Default::default(),
                            source: md.to_string(),
                        },
                    },
                    code,
                ]
            })
            .collect();

        Ok(Notebook {
            metadata: self.notebook_meta,
            nbformat: 4,
            nbformat_minor: 5,
            cells,
        })
    }
}

const CODE_SPLIT: &str = "--+code+--";

impl AstVisitor for NotebookWriter<'_> {
    fn visit_inline(&mut self, inline: &mut Inline) -> Result<()> {
        match inline {
            Inline::CodeBlock {
                source,
                tags,
                display_cell,
                global_idx,
                pos,
            } => {
                let rendered = source.to_string(true)?; // TODO: fix solution
                self.code_cells.push(Cell::Code {
                    common: CellCommon {
                        metadata: Default::default(),
                        source: rendered,
                    },
                    execution_count: None,
                    outputs: vec![], // TODO: fix outputs
                });

                *inline = Inline::Text(CODE_SPLIT.to_string());
            }
            _ => {}
        }
        Ok(())
    }
}
