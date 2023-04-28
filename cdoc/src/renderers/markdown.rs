use crate::ast::{math_block_md, Ast, Block, Inline, Shortcode};
use crate::config::OutputFormat;
use crate::document::{Document, DocumentMetadata};
use crate::parsers::shortcodes::ShortCodeDef;
use crate::renderers;
use crate::renderers::{add_args, RenderContext, RenderResult, Renderer};
use crate::templates::{TemplateContext, TemplateManager};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::max;
use std::collections::{BTreeMap, HashMap};
use tera::Tera;

#[derive(Serialize, Deserialize)]
pub struct MarkdownRenderer;

#[typetag::serde(name = "renderer_config")]
impl Renderer for MarkdownRenderer {
    fn render(&self, doc: &Document<Ast>, ctx: &RenderContext) -> Result<Document<RenderResult>> {
        let mut ctx = ToMarkdownContext {
            metadata: doc.metadata.clone(),
            ids: doc.ids.clone(),
            ids_map: doc.id_map.clone(),
            templates: ctx.templates.clone(),
            extra_args: ctx.extra_args.clone(),
            list_idx: None,
            list_lvl: 0,
        };

        let content = doc.content.0.clone().to_markdown(&mut ctx)?;

        Ok(Document {
            content,
            metadata: doc.metadata.clone(),
            variables: doc.variables.clone(),
            ids: doc.ids.clone(),
            id_map: doc.id_map.clone(),
        })
    }
}

pub struct ToMarkdownContext {
    pub metadata: DocumentMetadata,
    pub ids: HashMap<String, (usize, Vec<ShortCodeDef>)>,
    pub ids_map: HashMap<String, (usize, ShortCodeDef)>,
    pub templates: TemplateManager,
    pub extra_args: TemplateContext,
    pub list_idx: Option<usize>,
    pub list_lvl: usize,
}

pub trait ToMarkdown {
    fn to_markdown(self, ctx: &mut ToMarkdownContext) -> Result<String>;
}

impl ToMarkdown for Vec<Inline> {
    fn to_markdown(self, ctx: &mut ToMarkdownContext) -> Result<String> {
        self.into_iter().map(|i| i.to_markdown(ctx)).collect()
    }
}

impl ToMarkdown for Vec<Block> {
    fn to_markdown(self, ctx: &mut ToMarkdownContext) -> Result<String> {
        self.into_iter().map(|b| b.to_markdown(ctx)).collect()
    }
}

impl ToMarkdown for Inline {
    fn to_markdown(self, ctx: &mut ToMarkdownContext) -> Result<String> {
        match self {
            Inline::Text(s) => Ok(s),
            Inline::Emphasis(i) => Ok(format!("*{}*", i.to_markdown(ctx)?)),
            Inline::Strong(i) => Ok(format!("**{}**", i.to_markdown(ctx)?)),
            Inline::Strikethrough(i) => Ok(format!("~~{}~~", i.to_markdown(ctx)?)),
            Inline::Code(c) => Ok(format!("`{c}`")),
            Inline::SoftBreak => Ok("\n".to_string()),
            Inline::HardBreak => Ok("\n\n".to_string()),
            Inline::Rule => Ok("---\n".to_string()),
            Inline::Image(_tp, url, title, _) => Ok(format!("![{title}]({url})")),
            Inline::Link(_tp, url, title, _) => Ok(format!("[{title}]({url})")),
            Inline::Html(s) => Ok(s),
            Inline::Math(s, display_mode, trailing_space) => {
                Ok(math_block_md(&s, display_mode, trailing_space))
            }
            Inline::Shortcode(s) => render_shortcode_template(ctx, s),
        }
    }
}

impl ToMarkdown for Block {
    fn to_markdown(self, ctx: &mut ToMarkdownContext) -> Result<String> {
        match self {
            Block::Heading { lvl, inner, .. } => Ok(format!(
                "{} {}\n",
                "#".repeat(lvl as usize),
                inner.to_markdown(ctx)?
            )),
            Block::Plain(i) => Ok(i.to_markdown(ctx)?),
            Block::Paragraph(i) => Ok(format!("{}\n", i.to_markdown(ctx)?)),
            Block::BlockQuote(i) => Ok(i.to_markdown(ctx)?),
            Block::CodeBlock { source, .. } => Ok(format!("```\n{}\n```", source)),
            Block::List(idx, items) => {
                let inner: Result<String> = items.into_iter().map(|b| b.to_markdown(ctx)).collect();
                let inner = inner?;

                ctx.list_lvl += 1;

                let res = match idx {
                    None => renderers::render_value_template(
                        &ctx.templates,
                        "b_list_unordered",
                        OutputFormat::Markdown,
                        inner,
                    )?,
                    Some(start) => {
                        let mut args = TemplateContext::new();
                        args.insert("start", &start);
                        args.insert("value", &inner);
                        ctx.templates
                            .render("b_list_ordered", OutputFormat::Markdown, &args)?
                    }
                };
                ctx.list_lvl -= 1;
                Ok(res)
            }
            Block::ListItem(inner) => {
                let mut args = TemplateContext::new();
                args.insert("idx", &ctx.list_idx);
                if let Some(v) = ctx.list_idx.as_mut() {
                    *v += 1;
                }
                args.insert("value", &inner.to_markdown(ctx)?);
                Ok(format!(
                    "{}{}\n",
                    "\t".repeat(ctx.list_lvl), // Todo: Strange
                    ctx.templates
                        .render("b_list_item", OutputFormat::Markdown, &args)?
                ))
            }
        }
    }
}

fn render_params(
    parameters: HashMap<String, Vec<Block>>,
    ctx: &mut ToMarkdownContext,
) -> Result<HashMap<String, String>> {
    parameters
        .into_iter()
        .map(|(k, v)| Ok((k, v.to_markdown(ctx)?.trim().to_string())))
        .collect()
}

fn render_shortcode_template(ctx: &mut ToMarkdownContext, shortcode: Shortcode) -> Result<String> {
    let mut args = ctx.extra_args.clone();

    let name = match shortcode {
        Shortcode::Inline(def) => {
            let params = render_params(def.parameters, ctx)?;
            add_args(&mut args, def.id, def.num, &ctx.ids, &ctx.ids_map, params)?;
            def.name
        }
        Shortcode::Block(def, body) => {
            let params = render_params(def.parameters, ctx)?;
            add_args(&mut args, def.id, def.num, &ctx.ids, &ctx.ids_map, params)?;
            let body = body.to_markdown(ctx)?;
            args.insert("body", &body);
            def.name
        }
    };
    Ok(ctx.templates.render(&name, OutputFormat::Markdown, &args)?)
}

//
// struct MarkdownWriter<I> {
//     iter: I,
//     source: String,
//     list_order_num: Option<u64>,
// }
//
// impl<'a, I> MarkdownWriter<I>
// where
//     I: Iterator<Item = Event<'a>>,
// {
//     fn new(iter: I) -> Self {
//         MarkdownWriter {
//             iter,
//             source: String::new(),
//             list_order_num: None,
//         }
//     }
//
//     fn start_tag(&mut self, tag: Tag<'a>) {
//         match tag {
//             Tag::Paragraph => {}
//             Tag::Heading(level, _, _) => {
//                 let mut prefix = "#".repeat(heading_num(level));
//                 prefix.push(' ');
//                 self.source.push_str(&prefix);
//             }
//             Tag::BlockQuote => {}
//             Tag::CodeBlock(kind) => match kind {
//                 CodeBlockKind::Indented => {
//                     self.source.push_str("```plain\n");
//                 }
//                 CodeBlockKind::Fenced(cls) => {
//                     let s = cls.into_string();
//                     writeln!(self.source, "```{}", s).expect("Invalid format");
//                 }
//             },
//             Tag::List(i) => {
//                 self.list_order_num = i;
//             }
//             Tag::Item => match self.list_order_num {
//                 None => self.source.push_str("- "),
//                 Some(i) => {
//                     write!(self.source, "{}. ", i).expect("Invalid format");
//                     self.list_order_num = self.list_order_num.map(|i| i + 1);
//                 }
//             },
//             Tag::FootnoteDefinition(_) => {}
//             Tag::Table(_) => {}
//             Tag::TableHead => {}
//             Tag::TableRow => {}
//             Tag::TableCell => {}
//             Tag::Emphasis => self.source.push('*'),
//             Tag::Strong => self.source.push_str("__"),
//             Tag::Strikethrough => {}
//             Tag::Link(_, _, _) => self.source.push('['),
//             Tag::Image(_, _, _) => {}
//         }
//     }
//
//     fn end_tag(&mut self, tag: Tag<'a>) {
//         match tag {
//             Tag::CodeBlock(_) => self.source.push_str("\n```\n"),
//             Tag::Paragraph => self.source.push('\n'),
//             Tag::Heading(_, _, _) => self.source.push_str("\n\n"),
//             Tag::BlockQuote => {}
//             Tag::List(_) => self.source.push('\n'),
//             Tag::Item => self.source.push('\n'),
//             Tag::FootnoteDefinition(_) => {}
//             Tag::Table(_) => {}
//             Tag::TableHead => {}
//             Tag::TableRow => {}
//             Tag::TableCell => {}
//             Tag::Emphasis => self.source.push('*'),
//             Tag::Strong => self.source.push_str("__"),
//             Tag::Strikethrough => {}
//             Tag::Link(_type, dest, title) => {
//                 write!(self.source, "]({} {})", dest, title).expect("Invalid format");
//             }
//             Tag::Image(_, _, _) => {}
//         }
//     }
//
//     fn run(mut self) -> String {
//         while let Some(event) = self.iter.next() {
//             match event {
//                 Event::Start(tag) => self.start_tag(tag),
//                 Event::End(tag) => self.end_tag(tag),
//                 Event::Text(text) => {
//                     let ts = text.into_string();
//                     if &ts == "\\" {
//                         self.source.push_str("\\\\");
//                     } else {
//                         self.source.push_str(&ts)
//                     }
//                 }
//                 Event::Code(_) => {}
//                 Event::Html(text) => self.source.push_str(&text.into_string()),
//                 Event::FootnoteReference(_) => {}
//                 Event::SoftBreak => self.source.push('\n'),
//                 Event::HardBreak => self.source.push_str("\n\n"),
//                 Event::Rule => {}
//                 Event::TaskListMarker(_) => {}
//             };
//         }
//
//         self.source
//     }
// }
//
// pub fn render_markdown<'a, I>(iter: I) -> String
// where
//     I: Iterator<Item = Event<'a>>,
// {
//     MarkdownWriter::new(iter).run()
// }
//
