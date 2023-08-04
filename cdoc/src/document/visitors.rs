use crate::ast::{AstVisitor, CodeAttributes, Inline, Shortcode};
use crate::document::split_shortcodes;
use crate::notebook::CellOutput;
use crate::parsers::shortcodes::{parse_shortcode, ShortCodeCall};
use regex::Regex;
use std::collections::HashMap;
use std::ops::Range;
use std::str::FromStr;

pub struct MathInserter {
    math_blocks: Vec<Inline>,
}

impl MathInserter {
    pub fn new(math_blocks: Vec<Inline>) -> Self {
        MathInserter { math_blocks }
    }
}

impl MathInserter {
    fn replace_with_original(&mut self, source: &mut String) -> anyhow::Result<()> {
        let r = Regex::new(r"\*\*([0-9]+)\*\*")?;

        let mut out = String::new();
        let mut start_idx = 0;

        r.captures_iter(source).try_for_each(|m| {
            if let Some(ms) = m.get(1) {
                let idx = usize::from_str(ms.as_str())?;
                out.push_str(&source[start_idx..ms.range().start - 2]);

                if let Some(Inline::Math {
                    source,
                    display_block,
                    trailing_space,
                }) = self.math_blocks.get(idx)
                {
                    let trail = if *trailing_space { " " } else { "" };
                    if *display_block {
                        out.push_str(&format!("$${}$${}", source, trail));
                    } else {
                        out.push_str(&format!("${}${}", source, trail));
                    }
                }
                start_idx = ms.range().end + 2;
            }

            Ok::<(), anyhow::Error>(())
        })?;
        out.push_str(&source[start_idx..]);
        *source = out;
        Ok(())
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

    fn visit_code_block(
        &mut self,
        source: &mut String,
        _reference: &mut Option<String>,
        _attr: &mut CodeAttributes,
        _tags: &mut Option<Vec<String>>,
        _outputs: &mut Vec<CellOutput>,
        _display_cell: &mut bool,
    ) -> anyhow::Result<()> {
        self.replace_with_original(source)
    }
}

pub struct ShortcodeSourceDescriptor<'a> {
    call_src: &'a str,
    body_src: Option<&'a str>,
    call_range: Range<usize>,
    body_range: Option<Range<usize>>,
    cell: usize,
}

impl<'a> ShortcodeSourceDescriptor<'a> {
    pub fn new_inline(call_src: &'a str, call_range: Range<usize>, cell: usize) -> Self {
        ShortcodeSourceDescriptor {
            call_src,
            call_range,
            cell,
            body_src: None,
            body_range: None,
        }
    }

    pub fn new_body(
        call_src: &'a str,
        body_src: &'a str,
        call_range: Range<usize>,
        body_range: Range<usize>,
        cell: usize,
    ) -> Self {
        ShortcodeSourceDescriptor {
            call_src,
            call_range,
            cell,
            body_src: Some(body_src),
            body_range: Some(body_range),
        }
    }
}

pub struct ShortcodeInserter<'a> {
    shortcodes: Vec<ShortcodeSourceDescriptor<'a>>,
    counters: &'a mut HashMap<String, (usize, Vec<ShortCodeCall>)>,
}

impl<'a> ShortcodeInserter<'a> {
    pub fn new(
        shortcodes: Vec<ShortcodeSourceDescriptor<'a>>,
        counters: &'a mut HashMap<String, (usize, Vec<ShortCodeCall>)>,
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
                let descriptor = &self.shortcodes[idx];
                let code = parse_shortcode(descriptor.call_src)?;

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

                let base = code.into_base(
                    descriptor.call_range.clone(),
                    descriptor.cell,
                    self.counters,
                )?;
                let code = if let (Some(body_src), Some(body_range)) =
                    (descriptor.body_src, &descriptor.body_range)
                {
                    let body_blocks = split_shortcodes(
                        body_src,
                        body_range.start,
                        descriptor.cell,
                        self.counters,
                    )?;
                    Shortcode::Block(base, body_blocks, body_range.clone())
                } else {
                    Shortcode::Inline(base)
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
        _display_cell: &mut bool,
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

                if let Some(descriptor) = self.shortcodes.get(idx) {
                    if let (Some(body_src), Some(body_range)) =
                        (descriptor.body_src, &descriptor.body_range)
                    {
                        out.push_str(&format!("{{% {} %}}", descriptor.call_src));
                        let code = parse_shortcode(descriptor.call_src)?;
                        out.push_str(body_src);
                        out.push_str(&format!("{{% end_{} %}}", code.name))
                    } else {
                        out.push_str(&format!("{{{{ {} }}}}", descriptor.call_src));
                    }
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
