use crate::ast::{Block, CodeBlock, Command, Inline, Math, Parameter, Style};
use crate::code_ast::types::{CodeContent, CodeElem};
use crate::common::PosInfo;
use crate::raw::CodeAttr;
use anyhow::Result;
use linked_hash_map::LinkedHashMap;

/// Implements the visitor pattern for the cdoc Ast type. Blanket implementations are provided so
/// implementors only have to implement the methods they need to modify.
pub trait AstVisitor {
    fn walk_ast(&mut self, ast: &mut Vec<Block>) -> Result<()> {
        self.visit_vec_block(ast)
    }

    fn walk_vec_block(&mut self, blocks: &mut Vec<Block>) -> Result<()> {
        blocks.iter_mut().try_for_each(|b| self.visit_block(b))
    }

    fn walk_block(&mut self, block: &mut Block) -> Result<()> {
        match *block {
            Block::Heading { .. } => Ok(()),
            Block::Plain(ref mut i) => self.visit_vec_inline(i),
            Block::Paragraph(ref mut is) | Block::BlockQuote(ref mut is) => {
                self.visit_vec_inline(is)
            }
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
            Inline::Styled(ref mut is, ref mut style) => self.visit_styled(is, style),
            Inline::Code(s) => self.visit_code(s),
            Inline::SoftBreak => Ok(()),
            Inline::HardBreak => Ok(()),
            Inline::Rule => Ok(()),
            Inline::Image(_tp, _url, _alt, _inner) => Ok(()),
            Inline::Link(_tp, _url, _alt, _inner) => Ok(()),
            Inline::Html(h) => self.visit_html_inline(h),
            Inline::Math(math) => self.visit_math(math),
            Inline::Command(cmd) => self.visit_command(cmd),
            Inline::CodeBlock(block) => self.visit_code_block(block),
        }
    }

    fn visit_html_inline(&mut self, _input: &str) -> Result<()> {
        Ok(())
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

    fn visit_styled(&mut self, inlines: &mut Vec<Inline>, _style: &mut Style) -> Result<()> {
        self.walk_vec_inline(inlines)
    }
    #[allow(clippy::too_many_arguments)]
    fn visit_code_block(&mut self, _block: &mut CodeBlock) -> Result<()> {
        Ok(())
    }

    fn visit_code(&mut self, _source: &mut String) -> Result<()> {
        Ok(())
    }

    fn visit_math(&mut self, _math: &mut Math) -> Result<()> {
        Ok(())
    }
    fn visit_math_inline(&mut self, _source: &mut String) -> Result<()> {
        Ok(())
    }

    fn visit_command(&mut self, cmd: &mut Command) -> Result<()> {
        if let Some(body) = &mut cmd.body {
            self.walk_vec_block(body)?;
        }
        Ok(())
    }
}
