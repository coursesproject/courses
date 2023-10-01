use anyhow::Result;
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Ast, CodeBlock, Inline};
use cdoc_parser::document::{CodeOutput, Document};
use cdoc_parser::notebook::{Cell, CellCommon, CellMeta, JupyterLabMeta, Notebook, NotebookMeta};

use cdoc_base::node::visitor::ElementVisitor;
use cdoc_base::node::{Element, Node};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufWriter;

use crate::renderers::extensions::RenderExtension;
use crate::renderers::newrenderer::{ElementRenderer, ElementRendererConfig};
use crate::renderers::{
    DocumentRenderer, RenderContext, RenderElement, RenderResult, RendererConfig,
};

pub struct NotebookRendererBuilder;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NotebookRenderer;

#[typetag::serde(name = "notebook")]
impl RendererConfig for NotebookRenderer {
    fn build(&self) -> Result<Box<dyn DocumentRenderer>> {
        Ok(Box::new(self.clone()))
    }
}

impl DocumentRenderer for NotebookRenderer {
    fn render_doc(
        &mut self,
        ctx: &mut RenderContext,
        extensions: Vec<Box<dyn RenderExtension>>,
    ) -> Result<Document<RenderResult>> {
        let renderer = ElementRenderer::new("")?;

        for mut ext in extensions {
            ext.process(ctx, &renderer)?;
        }

        let writer = NotebookWriter {
            notebook_meta: ctx.notebook_output_meta.clone(),
            outputs: ctx.doc.code_outputs.clone(),
            code_cells: vec![],
            ctx,
            renderer,
        };

        let notebook: Notebook = writer.convert(ctx.doc.content.clone())?;
        let output = serde_json::to_string_pretty(&notebook)
            .expect("Invalid notebook (this is a bug)")
            .into();

        Ok(Document {
            content: output,
            meta: ctx.doc.meta.clone(),
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
    pub outputs: HashMap<u64, CodeOutput>,
    pub code_cells: Vec<Cell>,
    pub ctx: &'a RenderContext<'a>,
    pub renderer: ElementRenderer<'a>,
}

impl NotebookWriter<'_> {
    fn convert(mut self, mut elements: Vec<Element>) -> Result<Notebook> {
        let cell_meta = CellMeta {
            jupyter: Some(JupyterLabMeta {
                outputs_hidden: None,
                source_hidden: Some(true),
            }),
            ..Default::default()
        };

        let import = Cell::Code {
            common: CellCommon {
                id: "css_setup".to_string(),
                metadata: cell_meta,
                source: r#"import requests
from IPython.core.display import HTML
HTML(f"""
<style>
@import "https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css";
</style>
""")"#
                    .to_string(),
            },
            execution_count: Some(0),
            outputs: vec![],
        };

        self.walk_elements(&mut elements)?;

        let mut buf = BufWriter::new(Vec::new());

        self.renderer.render(&elements, self.ctx, &mut buf)?;

        let out_str = String::from_utf8(buf.into_inner()?)?;

        let md_cells = out_str.split(CODE_SPLIT);

        let mut cells = vec![import];

        for (idx, md) in md_cells.enumerate() {
            cells.push(Cell::Markdown {
                common: CellCommon {
                    id: nanoid!(),
                    metadata: Default::default(),
                    source: md.to_string(),
                },
            });
            if let Some(code) = self.code_cells.get(idx) {
                cells.push(code.clone());
            }
        }

        Ok(Notebook {
            metadata: self.notebook_meta,
            nbformat: 4,
            nbformat_minor: 5,
            cells,
        })
    }
}

const CODE_SPLIT: &str = "--+code+--";

impl ElementVisitor for NotebookWriter<'_> {
    fn visit_element(&mut self, element: &mut Element) -> Result<()> {
        if let Element::Node(node) = element {
            if node.type_id == "code_block" {
                if node.attributes.contains_key("is_cell") {
                    let rendered = "temp".to_string();

                    self.code_cells.push(Cell::Code {
                        common: CellCommon {
                            id: nanoid!(),
                            metadata: Default::default(),
                            source: rendered.trim().to_string(),
                        },
                        execution_count: Some(0),
                        outputs: vec![], // TODO: fix outputs
                    });
                }
            }

            *element = Element::Plain(CODE_SPLIT.into())
        }
        Ok(())
    }
}

// impl AstVisitor for NotebookWriter<'_> {
//     fn visit_inline(&mut self, inline: &mut Inline) -> Result<()> {
//         if let Inline::CodeBlock(CodeBlock {
//             source, attributes, ..
//         }) = inline
//         {
//             if attributes.contains(&"cell".into()) {
//                 let rendered = source.to_string(
//                     self.ctx
//                         .doc
//                         .meta
//                         .code_solutions
//                         .unwrap_or(self.ctx.parser_settings.solutions),
//                 )?;
//
//                 self.code_cells.push(Cell::Code {
//                     common: CellCommon {
//                         id: nanoid!(),
//                         metadata: Default::default(),
//                         source: rendered.trim().to_string(),
//                     },
//                     execution_count: Some(0),
//                     outputs: vec![], // TODO: fix outputs
//                 });
//
//                 *inline = Inline::Text(CODE_SPLIT.into());
//             }
//         }
//
//         Ok(())
//     }
// }
