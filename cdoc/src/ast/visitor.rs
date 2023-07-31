use crate::ast::{Ast, Block, CodeAttributes, Inline, Shortcode, ShortcodeBase};
use crate::notebook::CellOutput;
use anyhow::Result;

/// Implements the visitor pattern for the cdoc Ast type. Blanket implementations are provided so
/// implementors only have to implement the methods they need to modify.
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
            Block::Plain(ref mut i) => self.visit_vec_inline(i),
            Block::Paragraph(ref mut is) | Block::BlockQuote(ref mut is) => {
                self.visit_vec_inline(is)
            }
            Block::CodeBlock {
                ref mut source,
                ref mut reference,
                ref mut attr,
                ref mut tags,
                ref mut outputs,
                ref mut display_cell,
            } => self.visit_code_block(source, reference, attr, tags, outputs, display_cell),
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
                ref mut trailing_space,
            } => self.visit_math(source, display_block, trailing_space),
            Inline::Shortcode(ref mut s) => self.visit_shortcode(s),
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

    fn visit_code_block(
        &mut self,
        source: &mut String,
        _reference: &mut Option<String>,
        _attr: &mut CodeAttributes,
        _tags: &mut Option<Vec<String>>,
        _outputs: &mut Vec<CellOutput>,
        _display_cell: &mut bool,
    ) -> Result<()> {
        self.visit_code(source)
    }

    fn visit_code(&mut self, _source: &mut String) -> Result<()> {
        Ok(())
    }

    fn visit_math(
        &mut self,
        _source: &mut String,
        _display_block: &mut bool,
        _trailing_space: &mut bool,
    ) -> Result<()> {
        Ok(())
    }
    fn visit_math_inline(&mut self, _source: &mut String) -> Result<()> {
        Ok(())
    }

    fn walk_shortcode(&mut self, shortcode: &mut Shortcode) -> Result<()> {
        match shortcode {
            Shortcode::Inline(ref mut s) => self.visit_shortcode_base(s),
            Shortcode::Block(ref mut s, ref mut blocks) => {
                self.visit_shortcode_base(s)?;
                self.walk_vec_block(blocks)
            }
        }
    }

    fn visit_shortcode_base(&mut self, _shortcode_base: &mut ShortcodeBase) -> Result<()> {
        Ok(())
    }

    fn visit_shortcode(&mut self, shortcode: &mut Shortcode) -> Result<()> {
        self.walk_shortcode(shortcode)
    }
}
