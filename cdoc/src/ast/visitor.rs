use crate::ast::{Ast, Block, CodeAttributes, Inline};
use crate::notebook::CellOutput;
use anyhow::Result;

pub trait AstVisitor {
    fn walk_ast(&mut self, ast: &mut Ast) -> Result<()> {
        self.visit_vec_block(&mut ast.0)
    }

    fn walk_vec_block(&mut self, blocks: &mut Vec<Block>) -> Result<()> {
        blocks.iter_mut().try_for_each(|b| self.visit_block(b))
    }

    fn walk_block(&mut self, block: &mut Block) -> Result<()> {
        match *block {
            Block::Heading { .. } => Ok(()),
            Block::Plain(ref mut i) => self.visit_inline(i),
            Block::Paragraph(ref mut is) | Block::BlockQuote(ref mut is) => {
                self.visit_vec_inline(is)
            }
            Block::CodeBlock {
                ref mut source,
                ref mut reference,
                ref mut attr,
                ref mut outputs,
            } => self.visit_code_block(source, reference, attr, outputs),
            Block::List(_, ref mut blocks) => self.visit_vec_block(blocks),
            Block::ListItem(ref mut blocks) => self.visit_vec_block(blocks),
        }
    }

    fn walk_vec_inline(&mut self, inlines: &mut Vec<Inline>) -> Result<()> {
        inlines.iter_mut().try_for_each(|i| self.visit_inline(i))
    }

    fn walk_inline(&mut self, inline: &mut Inline) -> Result<()> {
        match inline {
            Inline::Text(_) => Ok(()),
            Inline::Emphasis(ref mut is)
            | Inline::Strong(ref mut is)
            | Inline::Strikethrough(ref mut is) => self.visit_vec_inline(is),
            Inline::Code(_) => Ok(()),
            Inline::SoftBreak => Ok(()),
            Inline::HardBreak => Ok(()),
            Inline::Rule => Ok(()),
            Inline::Image(_tp, _url, _alt) => Ok(()),
            Inline::Link(_tp, _url, _alt) => Ok(()),
            Inline::Html(_) => Ok(()),
        }
    }

    fn visit_vec_block(&mut self, blocks: &mut Vec<Block>) -> Result<()> {
        self.walk_vec_block(blocks)
    }
    fn visit_block(&mut self, block: &mut Block) -> Result<()> {
        self.walk_block(block)
    }
    fn visit_vec_inline(&mut self, inlines: &mut Vec<Inline>) -> Result<()> {
        self.walk_vec_inline(inlines)
    }
    fn visit_inline(&mut self, inline: &mut Inline) -> Result<()> {
        self.walk_inline(inline)
    }

    fn visit_code_block(
        &mut self,
        source: &mut String,
        _reference: &mut Option<String>,
        _attr: &mut CodeAttributes,
        _outputs: &mut Vec<CellOutput>,
    ) -> Result<()> {
        self.visit_code(source)
    }

    fn visit_code(&mut self, _source: &mut String) -> Result<()> {
        Ok(())
    }
}
