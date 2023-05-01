use crate::ast::{AstVisitor, Inline, Shortcode};
use crate::document::split_shortcodes;
use crate::parsers::shortcodes::{parse_shortcode, ShortCodeDef};
use regex::Regex;
use std::collections::HashMap;
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

pub struct ShortcodeInserter<'a> {
    shortcodes: Vec<(&'a str, &'a str)>,
    counters: &'a mut HashMap<String, (usize, Vec<ShortCodeDef>)>,
}

impl<'a> ShortcodeInserter<'a> {
    pub fn new(
        shortcodes: Vec<(&'a str, &'a str)>,
        counters: &'a mut HashMap<String, (usize, Vec<ShortCodeDef>)>,
    ) -> Self {
        ShortcodeInserter {
            shortcodes,
            counters,
        }
    }
}

impl AstVisitor for ShortcodeInserter<'_> {
    fn visit_inline(&mut self, inline: &mut Inline) -> anyhow::Result<()> {
        if let Inline::Emphasis(inner) = inline {
            let s: String = inner.iter_mut().map(|i| i.to_string()).collect();

            if let Ok(idx) = usize::from_str(&s) {
                let (def, body) = self.shortcodes[idx].clone();
                let code = parse_shortcode(def)?;

                self.counters
                    .get_mut(&code.name)
                    .map(|v| {
                        v.0 += 1;
                        v.1.push(code.clone());
                    })
                    .unwrap_or_else(|| {
                        self.counters
                            .insert(code.name.clone(), (1, vec![code.clone()]));
                    });

                let base = code.into_base(&mut self.counters)?;
                let code = if body == "" {
                    Shortcode::Inline(base)
                } else {
                    let body_blocks = split_shortcodes(body, &mut self.counters)?;
                    Shortcode::Block(base, body_blocks)
                };
                *inline = Inline::Shortcode(code);
            }
        }

        self.walk_inline(inline)
    }

    // fn visit_code(&mut self, _source: &mut String) -> anyhow::Result<()> {
    //     let r = Regex::new(r"_[0-9]+_")?;
    //     for m in r.find_iter() {
    //         m.
    //     }
    // }
}
