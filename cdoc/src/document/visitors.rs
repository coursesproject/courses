use crate::ast::{AstVisitor, Inline, Shortcode};
use std::str::FromStr;

pub struct MathInserter {
    math_blocks: Vec<Inline>,
}

impl MathInserter {
    pub fn new(math_blocks: Vec<Inline>) -> Self {
        MathInserter { math_blocks }
    }
}

impl AstVisitor for MathInserter {
    fn visit_inline(&mut self, inline: &mut Inline) -> anyhow::Result<()> {
        if let Inline::Strong(inner) = inline {
            let s: String = inner.iter_mut().map(|i| i.to_string()).collect();

            if let Ok(idx) = usize::from_str(&s) {
                *inline = self.math_blocks[idx].clone()
            }
        }

        self.walk_inline(inline)
    }
}

pub struct ShortcodeInserter {
    shortcodes: Vec<Shortcode>,
}

impl ShortcodeInserter {
    pub fn new(shortcodes: Vec<Shortcode>) -> Self {
        ShortcodeInserter { shortcodes }
    }
}

impl AstVisitor for ShortcodeInserter {
    fn visit_inline(&mut self, inline: &mut Inline) -> anyhow::Result<()> {
        if let Inline::Emphasis(inner) = inline {
            let s: String = inner.iter_mut().map(|i| i.to_string()).collect();

            if let Ok(idx) = usize::from_str(&s) {
                *inline = Inline::Shortcode(self.shortcodes[idx].clone());
            }
        }

        self.walk_inline(inline)
    }
}
