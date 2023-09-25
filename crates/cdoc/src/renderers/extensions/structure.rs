use crate::renderers::extensions::{RenderExtension, RenderExtensionConfig};
use crate::renderers::generic::GenericRenderer;
use crate::renderers::{RenderContext, RenderElement};
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Block, Command};
use cowstr::CowStr;

use linked_hash_map::LinkedHashMap;

use serde::{Deserialize, Serialize};

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
    label: Option<CowStr>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ElemVal {
    Heading {
        value: CowStr,
    },
    Command {
        name: CowStr,
        parameters: LinkedHashMap<CowStr, CowStr>,
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
        visitor.walk_ast(&mut ctx.doc.content.blocks.clone())?;

        let tree = visitor.construct_element_tree()?;

        ctx.doc.meta.user_defined.insert(
            "tree_raw".to_string(),
            serde_json::to_value(visitor.elems.clone())?,
        );

        ctx.doc
            .meta
            .user_defined
            .insert("tree".to_string(), serde_json::to_value(&tree)?);

        Ok(())
    }
}

impl DocStructureVisitor<'_> {
    pub fn construct_element_tree(&self) -> anyhow::Result<Vec<Tree>> {
        let lvl = &self.elems.get(0).map(|e| e.lvl).unwrap_or_default();
        let (tree, _) = self.construct_element_tree_inner2(&self.elems, 0, *lvl);
        Ok(tree)
    }

    fn construct_element_tree_inner2(
        &self,
        elems: &[Elem],
        current_idx: usize,
        current_lvl: u8,
    ) -> (Vec<Tree>, usize) {
        let mut tree: Vec<Tree> = vec![];
        let mut current_idx = current_idx;

        while current_idx < elems.len() {
            let current = &elems[current_idx];

            if current.lvl > current_lvl {
                let (children, new_idx) =
                    self.construct_element_tree_inner2(elems, current_idx, current.lvl);

                let t = tree.last_mut().unwrap();
                t.children = children;

                current_idx = new_idx;
            } else if current.lvl == current_lvl {
                tree.push(Tree {
                    elem: current.clone(),
                    children: vec![],
                });
                current_idx += 1;
            } else {
                return (tree, current_idx + 1);
            }
        }

        (tree, current_idx)
    }

    fn construct_element_tree_inner<'a, I: Iterator<Item = &'a Elem>>(
        &self,
        previous: &Elem,
        iter: &mut I,
        current_level: u8,
    ) -> anyhow::Result<Vec<Tree>> {
        let mut current = vec![];
        let mut previous = previous;
        let mut final_val = true;
        while let Some(elem) = iter.next() {
            if elem.lvl < current_level {
                current.push(Tree {
                    elem: previous.clone(),
                    children: vec![],
                });
                previous = elem;
                break;
            } else if elem.lvl > current_level {
                let inner = self.construct_element_tree_inner(elem, iter, current_level + 1)?;
                current.push(Tree {
                    elem: previous.clone(),
                    children: inner,
                });
                final_val = false;
            } else if elem.lvl == current_level {
                current.push(Tree {
                    elem: previous.clone(),
                    children: vec![],
                });
            }
            previous = elem;
        }
        // if final_val {
        //     current.push(Tree {
        //         elem: previous.clone(),
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
                classes: _,
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

    fn visit_command(&mut self, cmd: &mut Command) -> anyhow::Result<()> {
        let params = self
            .renderer
            .render_params(cmd.parameters.clone(), self.ctx)?;
        let params = params
            .into_iter()
            .map(|p| (p.key.unwrap(), p.value))
            .collect();

        if self
            .base
            .config
            .included_commands
            .contains(&cmd.function.to_string())
        {
            // println!(
            //     "included: {:?}, {}",
            //     self.base.config.included_commands, &cmd.function
            // );
            self.elems.push(Elem {
                val: ElemVal::Command {
                    name: cmd.function.clone(),
                    parameters: params,
                },
                lvl: self.current_level,
                label: cmd.label.clone(),
            });
            self.current_level += 1;
            self.walk_command(&mut cmd.body)?;
            self.current_level -= 1;
        } else {
            self.walk_command(&mut cmd.body)?;
        }

        Ok(())
    }
}
