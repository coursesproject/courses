use crate::parsers::shortcodes::parse_shortcode;
use std::fmt::{Display, Formatter};
use pulldown_cmark::{Options, Parser};
use pulldown_cmark::html::push_html;
use tera::Tera;

pub enum OutputFormat {
    Markdown,
    Html,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Markdown => write!(f, "md"),
            OutputFormat::Html => write!(f, "html"),
        }
    }
}

enum ShortcodeInfo {
    Inline(usize, usize),
    Block {
        def: (usize, usize),
        end: (usize, usize),
    },
}

fn extract_block(start: usize, input: &str) -> Option<ShortcodeInfo> {
    let end = start + input[start..].find("%}")?;

    let end_block = end + (&input[end..]).find("{% end %}")?;

    Some(ShortcodeInfo::Block {
        def: (start, end),
        end: (end_block, end_block + 9),
    })
}

fn extract_inline(start: usize, input: &str) -> Option<ShortcodeInfo> {
    let end = start + 2 + input[start..].find("}}")?;
    Some(ShortcodeInfo::Inline(start, end))
}

fn find_shortcode(input: &str) -> Option<ShortcodeInfo> {
    let start_inline = input.find("{{");
    let start_block = input.find("{%");

    match start_inline {
        None => start_block.and_then(|start| extract_block(start, input)),
        Some(inline_start_idx) => match start_block {
            None => extract_inline(inline_start_idx, input),
            Some(block_start_idx) => {
                if inline_start_idx < block_start_idx {
                    extract_inline(inline_start_idx, input)
                } else {
                    extract_block(block_start_idx, input)
                }
            }
        },
    }
}

pub struct ShortCodeProcessor<'a> {
    tera: &'a Tera,
}

impl<'a> ShortCodeProcessor<'a> {
    pub fn new(tera: &'a Tera) -> Self {
        ShortCodeProcessor { tera }
    }

    fn render_inline_template(&self, shortcode: &str) -> anyhow::Result<String> {
        let code = parse_shortcode(shortcode)?;
        let mut context = tera::Context::new();
        let name = format!("{}.tera.html", code.name);
        for (k, v) in code.parameters {
            context.insert(k, &v);
        }
        Ok(self.tera.render(&name, &context)?)
    }

    fn render_block_template(&self, shortcode: &str, body: &str) -> anyhow::Result<String> {
        let code = parse_shortcode(shortcode)?;
        let mut context = tera::Context::new();
        let name = format!("{}.tera.html", code.name);
        for (k, v) in code.parameters {
            context.insert(k, &v);
        }


        let processed = ShortCodeProcessor::new(self.tera).process(&body)?;
        let parser = Parser::new_ext(&processed, Options::all());
        let mut html = String::new();
        push_html(&mut html, parser);


        context.insert("body", &html);
        Ok(self.tera.render(&name, &context)?)
    }

    pub fn process(&self, input: &str) -> anyhow::Result<String> {
        let mut rest = input;

        let mut result = String::new();

        while rest.len() > 0 {
            match find_shortcode(rest) {
                None => {
                    result.push_str(rest);
                    rest = "";
                }

                Some(info) => {
                    match info {
                        ShortcodeInfo::Inline(start, end) => {
                            let pre = &rest[..start];
                            let post = &rest[end..];
                            let tmp_name = (&rest[(start + 2)..(end - 2)]).trim();
                            println!("{}", tmp_name);

                            let res = self.render_inline_template(tmp_name)?;

                            result.push_str(pre);
                            result.push_str(&res);

                            rest = post; // Start next round after the current shortcode position
                        }
                        ShortcodeInfo::Block { def, end } => {
                            let pre = &rest[..def.0];
                            let post = &rest[(end.1)..];

                            let tmp_name = (&rest[(def.0 + 2)..(def.1)]).trim();
                            let body = (&rest[(def.1 + 2)..(end.0) - 2]).trim();

                            let res = self.render_block_template(tmp_name, body)?;

                            result.push_str(pre);
                            result.push_str(&res);
                            result.push('\n');

                            rest = post; // Start next round after the current shortcode position
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}
