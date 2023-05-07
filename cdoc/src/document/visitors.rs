use crate::ast::{AstVisitor, CodeAttributes, Inline, Shortcode};
use crate::document::split_shortcodes;
use crate::notebook::CellOutput;
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
                let (def, body) = self.shortcodes[idx];
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

                let base = code.into_base(self.counters)?;
                let code = if body.is_empty() {
                    Shortcode::Inline(base)
                } else {
                    let body_blocks = split_shortcodes(body, self.counters)?;
                    Shortcode::Block(base, body_blocks)
                };
                *inline = Inline::Shortcode(code);
            }
        }

        self.walk_inline(inline)
    }

    fn visit_code_block(
        &mut self,
        source: &mut String,
        _reference: &mut Option<String>,
        _attr: &mut CodeAttributes,
        _tags: &mut Option<Vec<String>>,
        _outputs: &mut Vec<CellOutput>,
    ) -> anyhow::Result<()> {
        self.replace_with_original(source)
    }

    fn visit_code(&mut self, source: &mut String) -> anyhow::Result<()> {
        self.replace_with_original(source)
    }
}

impl ShortcodeInserter<'_> {
    fn replace_with_original(&mut self, source: &mut String) -> anyhow::Result<()> {
        let r = Regex::new(r"_([0-9]+)_")?;

        let mut out = String::new();
        let mut start_idx = 0;

        r.captures_iter(source).try_for_each(|m| {
            if let Some(ms) = m.get(1) {
                let idx = usize::from_str(ms.as_str())?;
                out.push_str(&source[start_idx..ms.range().start - 1]);

                let (def, body) = self.shortcodes[idx];

                if body.is_empty() {
                    out.push_str(&format!("{{{{ {} }}}}", def));
                } else {
                    out.push_str(&format!("{{% {} %}}", def));
                    let code = parse_shortcode(def)?;
                    out.push_str(body);
                    out.push_str(&format!("{{% end_{} %}}", code.name))
                }
                start_idx = ms.range().end + 1;
            }

            Ok::<(), anyhow::Error>(())
        })?;
        out.push_str(&source[start_idx..]);
        *source = out;
        Ok(())
    }
}
