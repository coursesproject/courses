use crate::ast::{AstVisitor, Block, Inline, Shortcode};
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

pub struct MathInserter {
    math_blocks: Vec<Inline>,
}

impl MathInserter {
    pub fn new(math_blocks: Vec<Inline>) -> Self {
        MathInserter { math_blocks }
    }
}

// lazy_static! {
//     static ref PATTERN: Regex = Regex::new(r"<([0-9]+)>").unwrap();
// }

impl AstVisitor for MathInserter {
    fn visit_inline(&mut self, inline: &mut Inline) -> anyhow::Result<()> {
        if let Inline::Strong(inner) = inline {
            let s: String = inner.into_iter().map(|i| i.to_string()).collect();

            match usize::from_str(&s) {
                Ok(idx) => *inline = self.math_blocks[idx].clone(),
                _ => {}
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
            let s: String = inner.into_iter().map(|i| i.to_string()).collect();

            match usize::from_str(&s) {
                Ok(idx) => *inline = Inline::Shortcode(self.shortcodes[idx].clone()),
                _ => {}
            }
        }

        self.walk_inline(inline)
    }
}
