use anyhow::Result;

use cdoc_parser::notebook::{Cell, CellCommon, CellMeta, JupyterLabMeta, Notebook, NotebookMeta};

use cdoc_base::document::CodeOutput;
use cdoc_base::node::visitor::NodeVisitor;
use cdoc_base::node::{Compound, Node};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufWriter;

use crate::renderers::base::{ElementRenderer, ElementRendererConfig};
use crate::renderers::extensions::RenderExtension;
use crate::renderers::{
    DocumentRenderer, RenderContext, RenderElement, RenderResult, RendererConfig,
};

pub struct NotebookRendererBuilder;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NotebookRenderer;

pub struct NotebookWriter<'a> {
    pub notebook_meta: NotebookMeta,
    pub outputs: HashMap<u64, CodeOutput>,
    pub code_cells: Vec<Cell>,
    pub ctx: &'a mut RenderContext<'a>,
    pub renderer: ElementRenderer<'a>,
}

impl NotebookWriter<'_> {
    fn convert(mut self, mut elements: Vec<Node>) -> Result<Notebook> {
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

impl NodeVisitor for NotebookWriter<'_> {
    fn visit_element(&mut self, element: &mut Node) -> Result<()> {
        if let Node::Compound(node) = element {
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

            *element = Node::Plain(CODE_SPLIT.into())
        }
        Ok(())
    }
}
