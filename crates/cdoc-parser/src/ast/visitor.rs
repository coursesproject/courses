use crate::ast::{Block, Command, Inline, Parameter, Style};
use crate::code_ast::types::CodeContent;
use crate::common::PosInfo;
use anyhow::Result;

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
            Inline::Math {
                ref mut source,
                ref mut display_block,
                ref mut pos,
            } => self.visit_math(source, display_block, pos),
            Inline::Command(Command {
                function,
                id,
                parameters,
                body,
                pos,
                global_idx,
            }) => self.visit_command(function, id, parameters, body, pos, global_idx),
            Inline::CodeBlock {
                source,
                tags,
                display_cell,
                global_idx,
                pos,
            } => self.visit_code_block(source, tags, display_cell, global_idx, pos),
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
    fn visit_code_block(
        &mut self,
        _source: &mut CodeContent,
        _tags: &mut Option<Vec<String>>,
        _display_cell: &mut bool,
        _global_idx: &mut usize,
        _pos: &mut PosInfo,
    ) -> Result<()> {
        Ok(())
    }

    fn visit_code(&mut self, _source: &mut String) -> Result<()> {
        Ok(())
    }

    fn visit_math(
        &mut self,
        _source: &mut String,
        _display_block: &mut bool,
        _pos: &mut PosInfo,
    ) -> Result<()> {
        Ok(())
    }
    fn visit_math_inline(&mut self, _source: &mut String) -> Result<()> {
        Ok(())
    }

    fn visit_command(
        &mut self,
        _function: &mut String,
        _id: &mut Option<String>,
        _parameters: &mut Vec<Parameter>,
        body: &mut Option<Vec<Block>>,
        _pos: &mut PosInfo,
        _global_idx: &mut usize,
    ) -> Result<()> {
        if let Some(body) = body {
            self.walk_vec_block(body)?;
        }
        Ok(())
    }
}
