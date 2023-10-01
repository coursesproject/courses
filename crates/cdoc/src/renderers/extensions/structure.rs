use crate::renderers::extensions::{RenderExtension, RenderExtensionConfig};
use crate::renderers::{RenderContext, RenderElement};
use cdoc_parser::ast::visitor::AstVisitor;
use cdoc_parser::ast::{Block, Command};
use cowstr::CowStr;
use std::cmp::Ordering;
use std::collections::HashMap;

use linked_hash_map::LinkedHashMap;

use crate::renderers::newrenderer::ElementRenderer;
use serde::{Deserialize, Serialize};
//
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct DocStructureConfig {
//     pub max_heading_level: usize,
//     pub included_commands: Vec<String>,
// }
//
// #[typetag::serde(name = "doc_structure")]
// impl RenderExtensionConfig for DocStructureConfig {
//     fn build(&self) -> anyhow::Result<Box<dyn RenderExtension>> {
//         Ok(Box::new(DocStructure {
//             config: self.clone(),
//         }))
//     }
// }
//
// pub struct DocStructure {
//     config: DocStructureConfig,
// }
//
// pub struct DocStructureVisitor<'a> {
//     base: &'a DocStructure,
//     ctx: &'a RenderContext<'a>,
//     elems: Vec<Elem>,
//     current_level: u8,
//     renderer: ElementRenderer<'a>,
//     num_counters: HashMap<String, usize>,
// }
//
// impl<'a> DocStructureVisitor<'a> {
//     pub fn new(
//         base: &'a DocStructure,
//         ctx: &'a RenderContext<'a>,
//         renderer: ElementRenderer<'a>,
//     ) -> Self {
//         DocStructureVisitor {
//             base,
//             ctx,
//             elems: vec![],
//             current_level: 0,
//             renderer,
//             num_counters: HashMap::default(),
//         }
//     }
// }
//
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Elem {
//     #[serde(flatten)]
//     val: ElemVal,
//     lvl: u8,
//     label: Option<CowStr>,
//     num: usize,
//     chrono_num: usize,
// }
//
// #[derive(Debug, Serialize, Deserialize, Clone)]
// #[serde(tag = "type", rename_all = "snake_case")]
// pub enum ElemVal {
//     Heading {
//         value: CowStr,
//     },
//     Command {
//         name: CowStr,
//         parameters: LinkedHashMap<CowStr, CowStr>,
//     },
//     CodeBlock,
// }
//
// impl ElemVal {
//     pub fn type_id(&self) -> &str {
//         match self {
//             ElemVal::Heading { .. } => "heading",
//             ElemVal::Command { name, .. } => name.as_str(),
//             ElemVal::CodeBlock => "code",
//         }
//     }
// }
//
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct Tree {
//     elem: Elem,
//     children: Vec<Tree>,
// }
//
// impl RenderExtension for DocStructure {
//     fn name(&self) -> String {
//         "Document structure".to_string()
//     }
//
//     fn process(
//         &mut self,
//         ctx: &mut RenderContext,
//         renderer: ElementRenderer,
//     ) -> anyhow::Result<()> {
//         let mut visitor = DocStructureVisitor::new(self, ctx, renderer);
//         visitor.walk_ast(&mut ctx.doc.content.blocks.clone())?;
//
//         let tree = visitor.construct_element_tree()?;
//
//         ctx.doc.meta.user_defined.insert(
//             "tree_raw".to_string(),
//             serde_json::to_value(visitor.elems.clone())?,
//         );
//
//         ctx.doc
//             .meta
//             .user_defined
//             .insert("tree".to_string(), serde_json::to_value(&tree)?);
//
//         Ok(())
//     }
// }
//
// //noinspection RsExternalLinter
// fn construct_element_tree_inner2(
//     elems: &[Elem],
//     current_idx: usize,
//     current_lvl: u8,
// ) -> (Vec<Tree>, usize) {
//     let mut tree: Vec<Tree> = vec![];
//     let mut current_idx = current_idx;
//     let mut current_counters = HashMap::new();
//
//     while current_idx < elems.len() {
//         let current = &elems[current_idx];
//
//         match current.lvl.cmp(&current_lvl) {
//             Ordering::Greater => {
//                 let (children, new_idx) =
//                     construct_element_tree_inner2(elems, current_idx, current.lvl);
//
//                 let t = tree.last_mut().unwrap();
//                 t.children = children;
//
//                 current_idx = new_idx;
//             }
//             Ordering::Equal => {
//                 let mut elem = current.clone();
//                 let cnum = current_counters
//                     .entry(elem.val.type_id().to_string())
//                     .or_insert(1);
//                 elem.num = *cnum;
//                 *cnum += 1;
//
//                 tree.push(Tree {
//                     elem,
//                     children: vec![],
//                 });
//                 current_idx += 1;
//             }
//             Ordering::Less => return (tree, current_idx),
//         }
//     }
//
//     (tree, current_idx)
// }
//
// impl DocStructureVisitor<'_> {
//     pub fn construct_element_tree(&self) -> anyhow::Result<Vec<Tree>> {
//         let lvl = &self.elems.get(0).map(|e| e.lvl).unwrap_or(1);
//         let (tree, _) = construct_element_tree_inner2(&self.elems, 0, *lvl);
//         Ok(tree)
//     }
// }
//
// impl AstVisitor for DocStructureVisitor<'_> {
//     fn visit_block(&mut self, block: &mut Block) -> anyhow::Result<()> {
//         if let Block::Heading {
//             lvl,
//             id,
//             classes: _,
//             inner,
//         } = block
//         {
//             let inner = self.renderer.render_inner(inner, self.ctx)?;
//             let cnum = self.num_counters.entry("heading".to_string()).or_insert(1);
//
//             self.elems.push(Elem {
//                 val: ElemVal::Heading { value: inner },
//                 lvl: *lvl,
//                 label: id.clone(),
//                 num: 0,
//                 chrono_num: *cnum,
//             });
//
//             *cnum += 1;
//             self.current_level = *lvl + 1;
//         }
//
//         self.walk_block(block)
//     }
//
//     fn visit_command(&mut self, cmd: &mut Command) -> anyhow::Result<()> {
//         let params = self
//             .renderer
//             .render_params(cmd.parameters.clone(), self.ctx)?;
//         let params = params
//             .into_iter()
//             .map(|p| (p.key.unwrap(), p.value))
//             .collect();
//
//         if self
//             .base
//             .config
//             .included_commands
//             .contains(&cmd.function.to_string())
//         {
//             let cnum = self
//                 .num_counters
//                 .entry(cmd.function.to_string())
//                 .or_insert(1);
//
//             self.elems.push(Elem {
//                 val: ElemVal::Command {
//                     name: cmd.function.clone(),
//                     parameters: params,
//                 },
//                 lvl: self.current_level,
//                 label: cmd.label.clone(),
//                 num: 0,
//                 chrono_num: *cnum,
//             });
//             *cnum += 1;
//
//             self.current_level += 1;
//             self.walk_command(&mut cmd.body)?;
//             self.current_level -= 1;
//         } else {
//             self.walk_command(&mut cmd.body)?;
//         }
//
//         Ok(())
//     }
// }
