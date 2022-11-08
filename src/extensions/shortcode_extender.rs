use crate::parsers::shortcodes::{parse_shortcode, ShortCode, ShortCodeParser};
use anyhow::anyhow;
use pulldown_cmark::{CowStr, Event};
use std::cmp::min;
use std::collections::LinkedList;
use std::fmt::{Display, Formatter};
use std::iter::zip;
use tera::Tera;
//
// pub struct ShortCodeExtender<'a, I> {
//     event_queue: LinkedList<Event<'a>>,
//     block_stack: LinkedList<ShortCodeTag>,
//     event_iter: I,
//     tera: &'a Tera,
// }
//
// static SHORTCODE_TAG_START_INLINE: &str = "{{";
// static SHORTCODE_TAG_END_INLINE: &str = "}}";
// static SHORTCODE_TAG_START_BLOCK: &str = "{%";
// static SHORTCODE_TAG_END_BLOCK: &str = "%}";
//
// struct ShortCodeTag {
//     start: usize,
//     end: usize,
//     typ: TagType,
// }
//
// impl ShortCodeTag {
//     fn get_inner<'a>(&'a self, input: &'a str) -> &str {
//         &input[(self.start + 2)..(self.end - 2)]
//     }
//
//     fn get_between<'a>(&'a self, other: &'a Self, input: &'a str) -> &str {
//         &input[self.end..other.start]
//     }
// }
//
// enum TagType {
//     Inline,
//     Block,
// }
//
// fn split_string_by_tags<'a>(input: &'a str, tags: &'a Vec<ShortCodeTag>) -> Vec<&'a str> {
//     let mut res = vec![&input[..tags[0].start]];
//
//     for i in 0..(tags.len() - 1) {
//         let strip = tags[i].get_between(&tags[i + 1], input);
//         res.push(strip);
//     }
//
//     res
// }
//
// fn find_shortcode_tags(input: &str) -> anyhow::Result<Vec<ShortCodeTag>> {
//     let mut rest = input;
//     let mut ret = Vec::new();
//     let mut offset = 0;
//
//     while rest.len() > 0 {
//         let inline = rest.find(SHORTCODE_TAG_START_INLINE);
//         let block = rest.find(SHORTCODE_TAG_END_BLOCK);
//
//         println!("{}", rest);
//         let end = match (inline, block) {
//             (Some(inline_start_idx), Some(block_start_idx)) => {
//                 if inline_start_idx < block_start_idx {
//                     let end = rest
//                         .find(SHORTCODE_TAG_END_INLINE)
//                         .ok_or(anyhow!("Missing inline end tag"))?;
//                     ret.push(ShortCodeTag {
//                         start: offset + inline_start_idx,
//                         end: offset + end + 2,
//                         typ: TagType::Inline,
//                     });
//                     end + 2
//                 } else {
//                     let end = rest
//                         .find(SHORTCODE_TAG_END_BLOCK)
//                         .ok_or(anyhow!("Missing block end tag"))?;
//                     ret.push(ShortCodeTag {
//                         start: offset + block_start_idx,
//                         end: offset + end + 2,
//                         typ: TagType::Block,
//                     });
//                     end + 2
//                 }
//             }
//             (Some(inline_start_idx), None) => {
//                 let end = rest
//                     .find(SHORTCODE_TAG_END_INLINE)
//                     .ok_or(anyhow!("Missing inline end tag"))?;
//                 ret.push(ShortCodeTag {
//                     start: offset + inline_start_idx,
//                     end: offset + end + 2,
//                     typ: TagType::Inline,
//                 });
//                 end + 2
//             }
//             (None, Some(block_start_idx)) => {
//                 let end = rest
//                     .find(SHORTCODE_TAG_END_BLOCK)
//                     .ok_or(anyhow!("Missing block end tag"))?;
//                 ret.push(ShortCodeTag {
//                     start: offset + block_start_idx,
//                     end: offset + end + 2,
//                     typ: TagType::Block,
//                 });
//                 end + 2
//             }
//             (None, None) => rest.len(),
//         };
//
//         rest = &rest[end..];
//         offset += end;
//     }
//
//     Ok(ret)
// }
//
// impl<'a, I> ShortCodeExtender<'a, I> {
//     pub fn new(tera: &'a Tera, iter: I) -> Self {
//         ShortCodeExtender {
//             event_queue: LinkedList::new(),
//             block_stack: LinkedList::new(),
//             event_iter: iter,
//             tera,
//         }
//     }
//
//     fn render_inline_template(&self, shortcode: &str) -> anyhow::Result<String> {
//         let code = parse_shortcode(shortcode)?;
//         let mut context = tera::Context::new();
//         let name = format!("{}.tera.html", code.name);
//         for (k, v) in code.parameters {
//             context.insert(k, &v);
//         }
//         Ok(self.tera.render(&name, &context)?)
//     }
// }
//
// impl<'a, I> Iterator for ShortCodeExtender<'a, I>
// where
//     I: Iterator<Item = Event<'a>>,
// {
//     type Item = Event<'a>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.event_queue.len() == 0 {
//             let event = self.event_iter.next()?;
//             match event {
//                 Event::Text(txt) => {
//                     let tags = find_shortcode_tags(&txt).expect("Couldn't find shortcode");
//                     if tags.len() == 0 {
//                         return Some(Event::Text(txt));
//                     }
//
//                     let strips = split_string_by_tags(&txt, &tags);
//
//                     for (strip, tag) in zip(strips, &tags) {
//                         match tag.typ {
//                             TagType::Inline => {
//                                 self.event_queue.push_front(Event::Text(CowStr::Boxed(
//                                     strip.to_string().into_boxed_str(),
//                                 )));
//                                 let html = self
//                                     .render_inline_template(tag.get_inner(&txt))
//                                     .expect("Render failed");
//                                 self.event_queue
//                                     .push_front(Event::Html(CowStr::Boxed(html.into_boxed_str())));
//                             }
//                             TagType::Block => {}
//                         }
//                     }
//
//                     self.event_queue.push_front(Event::Text(CowStr::Boxed(
//                         (&txt)[tags.last().unwrap().end..]
//                             .to_string()
//                             .into_boxed_str(),
//                     )));
//
//                     self.event_queue.pop_back()
//                 }
//                 _ => Some(event),
//             }
//         } else {
//             self.event_queue.pop_back()
//         }
//     }
// }

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
        end: (end_block, end_block + 11),
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
        context.insert("body", body);
        Ok(self.tera.render(&name, &context)?)
    }

    pub fn process(&self, input: &str) -> String {
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

                            let res = match self.render_inline_template(tmp_name) {
                                Ok(res) => res,
                                Err(e) => e.to_string(),
                            };

                            result.push_str(pre);
                            result.push_str(&res);

                            rest = post; // Start next round after the current shortcode position
                        }
                        ShortcodeInfo::Block { def, end } => {
                            let pre = &rest[..def.0];
                            let post = &rest[(end.1)..];

                            let tmp_name = (&rest[(def.0 + 2)..(def.1)]).trim();
                            let body = (&rest[(def.1 + 2)..(end.0) - 2]).trim();

                            let res = match self.render_block_template(tmp_name, body) {
                                Ok(res) => res,
                                Err(e) => e.to_string(),
                            };

                            result.push_str(pre);
                            result.push_str(&res);

                            rest = post; // Start next round after the current shortcode position
                        }
                    }
                }
            }
        }

        result
    }
}
