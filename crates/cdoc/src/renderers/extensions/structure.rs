use crate::parser::ParserSettings;
use crate::preprocessors::cell_outputs::CellProcessor;
use crate::preprocessors::{AstPreprocessor, AstPreprocessorConfig, Error, PreprocessorContext};
use crate::renderers::extensions::{RenderExtension, RenderExtensionConfig};
use crate::renderers::generic::GenericRenderer;
use crate::renderers::{RenderContext, RenderElement};
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Ast, Block, Command, Inline};
use cdoc_parser::document::Document;
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocStructureConfig {
    pub max_heading_level: usize,
    pub included_commands: Vec<String>,
}

#[typetag::serde(name = "doc_structure")]
impl RenderExtensionConfig for DocStructureConfig {
    fn build(&self) -> anyhow::Result<Box<dyn RenderExtension>> {
        Ok(Box::new(DocStructure {
            config: self.clone(),
        }))
    }
}

pub struct DocStructure {
    config: DocStructureConfig,
}

pub struct DocStructureVisitor<'a> {
    base: &'a DocStructure,
    ctx: &'a RenderContext<'a>,
    elems: Vec<Elem>,
    current_level: u8,
    renderer: GenericRenderer,
}

impl<'a> DocStructureVisitor<'a> {
    pub fn new(
        base: &'a DocStructure,
        ctx: &'a RenderContext<'a>,
        renderer: GenericRenderer,
    ) -> Self {
        DocStructureVisitor {
            base,
            ctx,
            elems: vec![],
            current_level: 0,
            renderer,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Elem {
    #[serde(flatten)]
    val: ElemVal,
    lvl: u8,
    label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ElemVal {
    Heading {
        value: String,
    },
    Command {
        name: String,
        parameters: LinkedHashMap<String, String>,
    },
    CodeBlock,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tree {
    elem: Elem,
    children: Vec<Tree>,
}

impl RenderExtension for DocStructure {
    fn name(&self) -> String {
        "Document structure".to_string()
    }

    fn process(
        &mut self,
        ctx: &mut RenderContext,
        renderer: GenericRenderer,
    ) -> anyhow::Result<()> {
        let mut visitor = DocStructureVisitor::new(&self, ctx, renderer);
        visitor.walk_ast(&mut ctx.doc.content.0.clone())?;

        let tree = visitor.construct_element_tree()?;

        ctx.doc
            .meta
            .user_defined
            .insert("tree".to_string(), serde_json::to_value(&tree)?);

        Ok(())
    }
}

impl DocStructureVisitor<'_> {
    pub fn construct_element_tree(&self) -> anyhow::Result<Vec<Tree>> {
        let mut iter = self.elems.iter();
        if let Some(elem) = iter.next() {
            self.construct_element_tree_inner(elem, &mut iter, 1)
        } else {
            Ok(vec![])
        }
    }

    fn construct_element_tree_inner<'a, I: Iterator<Item = &'a Elem>>(
        &self,
        elem: &Elem,
        iter: &mut I,
        current_level: u8,
    ) -> anyhow::Result<Vec<Tree>> {
        let mut current = vec![];
        let mut previous = elem;
        let mut final_push = true;
        while let Some(elem) = iter.next() {
            if elem.lvl < current_level {
                final_push = false;
                break;
            } else if elem.lvl > current_level {
                let inner = self.construct_element_tree_inner(elem, iter, current_level + 1)?;
                current.push(Tree {
                    elem: previous.clone(),
                    children: inner,
                });
            } else if elem.lvl == current_level {
                current.push(Tree {
                    elem: previous.clone(),
                    children: vec![],
                });
            }
            previous = elem;
        }
        // if final_push {
        //     current.push(Tree {
        //         elem: elem.clone(),
        //         children: vec![],
        //     });
        // }

        Ok(current)
    }
}

impl AstVisitor for DocStructureVisitor<'_> {
    fn visit_block(&mut self, block: &mut Block) -> anyhow::Result<()> {
        match block {
            Block::Heading {
                lvl,
                id,
                classes,
                inner,
            } => {
                let inner = self.renderer.render_inner(inner, self.ctx)?;
                self.elems.push(Elem {
                    val: ElemVal::Heading { value: inner },
                    lvl: *lvl,
                    label: id.clone(),
                });
                self.current_level = *lvl + 1;
            }
            _ => {}
        }

        self.walk_block(block)
    }

    fn walk_command(&mut self, body: &mut Option<Vec<Block>>) -> anyhow::Result<()> {
        if let Some(body) = body {
            self.current_level += 1;
            self.walk_vec_block(body)?;
            self.current_level -= 1;
        }
        Ok(())
    }

    fn visit_command(&mut self, cmd: &mut Command) -> anyhow::Result<()> {
        let params = self
            .renderer
            .render_params(cmd.parameters.clone(), self.ctx)?;
        let params = params
            .into_iter()
            .map(|p| (p.key.unwrap(), p.value))
            .collect();

        self.elems.push(Elem {
            val: ElemVal::Command {
                name: cmd.function.clone(),
                parameters: params,
            },
            lvl: self.current_level,
            label: cmd.label.clone(),
        });

        self.walk_command(&mut cmd.body)
    }
}
