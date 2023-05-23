use crate::ast::{Ast, Block};
use anyhow::Result;
use pulldown_cmark::HeadingLevel;
use replace_with::replace_with_or_abort;
use serde::{Deserialize, Serialize};

use crate::document::Document;
use crate::notebook::{Cell, CellCommon, Notebook, NotebookMeta};
use crate::renderers::generic::GenericRenderer;
use crate::renderers::{DocumentRenderer, RenderContext, RenderElement, RenderResult};

pub struct NotebookRendererBuilder;

#[derive(Serialize, Deserialize)]
pub struct NotebookRenderer;

impl DocumentRenderer for NotebookRenderer {
    fn render_doc(&mut self, ctx: &RenderContext) -> Result<Document<RenderResult>> {
        let renderer = GenericRenderer;

        let writer = NotebookWriter {
            cell_source: Vec::new(),
            finished_cells: vec![],
            ctx,
            notebook_meta: ctx.notebook_output_meta.clone(),
            renderer,
        };

        let notebook: Notebook = writer.convert(ctx.doc.content.clone())?;
        let output = serde_json::to_string(&notebook).expect("Invalid notebook (this is a bug)");

        Ok(Document {
            content: output,
            metadata: ctx.doc.metadata.clone(),
            variables: ctx.doc.variables.clone(),
            ids: ctx.doc.ids.clone(),
            id_map: ctx.doc.id_map.clone(),
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

struct NotebookWriter<'a> {
    cell_source: Vec<u8>,
    finished_cells: Vec<Cell>,
    notebook_meta: NotebookMeta,
    ctx: &'a RenderContext<'a>,
    renderer: GenericRenderer,
}

impl NotebookWriter<'_> {
    fn push_markdown_cell(&mut self) {
        replace_with_or_abort(&mut self.cell_source, |source| {
            self.finished_cells.push(Cell::Markdown {
                common: CellCommon {
                    metadata: Default::default(),
                    source: String::from_utf8(source).unwrap(),
                },
            });
            Vec::new()
        });
    }

    fn block(&mut self, block: &Block) -> Result<()> {
        match block {
            Block::CodeBlock { source, .. } => {
                self.push_markdown_cell();
                let c = Cell::Code {
                    common: CellCommon {
                        metadata: Default::default(),
                        source: source.clone(),
                    },
                    execution_count: None,
                    outputs: Vec::new(),
                };
                self.finished_cells.push(c);
            }
            _ => {
                self.renderer
                    .render(block, self.ctx, &mut self.cell_source)?;
            }
        }

        Ok(())
    }

    fn convert(mut self, ast: Ast) -> Result<Notebook> {
        for b in &ast.0 {
            self.block(b)?;
        }

        self.push_markdown_cell();

        Ok(Notebook {
            metadata: self.notebook_meta,
            nbformat: 4,
            nbformat_minor: 5,
            cells: self.finished_cells,
        })
    }
}
