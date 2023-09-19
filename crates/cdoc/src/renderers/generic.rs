use std::collections::HashMap;

use anyhow::{anyhow, Context as AhContext, Result};
use serde::{Deserialize, Serialize};

use cdoc_parser::ast::{Block, CodeBlock, Command, Inline, Math, Parameter, Style, Value};
use cdoc_parser::document::{CodeOutput, Document, Image, Outval};
use std::io::{Cursor, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use tera::Context;

use crate::renderers::{DocumentRenderer, RenderContext, RenderElement, RenderResult};
use crate::templates::{TemplateDefinition, TemplateType};

fn write_bytes(source: &str, mut buf: impl Write) -> Result<()> {
    let bytes = source.as_bytes();
    let l = buf.write(bytes)?;
    (l == bytes.len())
        .then_some(())
        .ok_or(anyhow!("did not write correct number of bytes"))
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct GenericRenderer {
    list_level: usize,
    current_list_idx: Vec<Option<u64>>,
    counters: HashMap<String, usize>,
}

#[typetag::serde(name = "generic")]
impl DocumentRenderer for GenericRenderer {
    fn render_doc(&mut self, ctx: &RenderContext) -> Result<Document<RenderResult>> {
        // let doc = doc.to_events();
        // let dd = doc.to_events();
        //
        // let mut output = String::new();
        // html::push_html(&mut output, dd);
        let buf = Vec::new();
        let mut cursor = Cursor::new(buf);
        self.render(&ctx.doc.content.0, ctx, &mut cursor)?;

        let content = String::from_utf8(cursor.get_ref().clone())?;
        Ok(Document {
            content,
            meta: ctx.doc.meta.clone(),
            code_outputs: ctx.doc.code_outputs.clone(),
        })
    }
}

pub struct RenderedParam {
    pub key: Option<String>,
    pub value: String,
}

impl GenericRenderer {
    fn render_params(
        &mut self,
        parameters: Vec<Parameter>,
        ctx: &RenderContext,
    ) -> Result<Vec<RenderedParam>> {
        parameters
            .iter()
            .map(|p| {
                let value: String = match &p.value {
                    Value::Flag(_) => String::new(),
                    Value::Content(blocks) => self.render_inner(blocks, ctx)?,
                    Value::String(s) => s.to_string(),
                };

                Ok(RenderedParam {
                    key: p.key.clone(),
                    value,
                })
            })
            .collect()
    }

    fn render_command_template(
        &mut self,
        ctx: &RenderContext,
        command: &Command,
        buf: impl Write,
    ) -> Result<()> {
        let mut args = ctx.extra_args.clone();
        args.insert("defs", &ctx.templates.definitions);
        args.insert("refs", &ctx.references);
        // println!("{:?}", &ctx.references);
        args.insert("refs_by_type", &ctx.references_by_type);

        let num = self.fetch_and_inc_num(command.function.clone(), &command.label);
        let rendered = self
            .render_params(command.parameters.clone(), ctx)
            .with_context(|| {
                format!(
                    "error rendering shortcode {} at position {} and global index {}",
                    command.function, command.pos.start, command.global_idx
                )
            })?;
        let tdef = ctx
            .templates
            .get_template(&command.function, TemplateType::Shortcode)
            .with_context(|| format!("at {}", command.pos.get_with_margin(100)))?;
        let r: Result<Vec<()>> = ctx
            .templates
            .validate_args_for_template(&command.function, &rendered)
            .with_context(|| {
                format!(
                    "validation error at position {} {}",
                    command.pos.start, command.global_idx
                )
            })?
            .into_iter()
            .collect();
        r?;

        add_args(&tdef, &mut args, &command.label, num, rendered)?;
        let body = command
            .body
            .as_ref()
            .map(|b| self.render_inner(b, ctx))
            .transpose()?;
        args.insert("body", &body);

        ctx.templates.render(
            &command.function,
            ctx.format.template_prefix(),
            TemplateType::Shortcode,
            &args,
            buf,
        )
    }

    fn fetch_and_inc_num(&mut self, typ: String, label: &Option<String>) -> usize {
        let num = if label.is_some() {
            let num = self.counters.entry(typ).or_insert(0);
            *num += 1;
            *num
        } else {
            0
        };
        num
    }

    fn render_math(
        &mut self,
        display_mode: bool,
        inner: &str,
        label: &Option<String>,
        ctx: &RenderContext,
        buf: impl Write,
    ) -> Result<()> {
        let mut args = Context::default();

        args.insert("label", label);
        args.insert("display_mode", &display_mode);
        if display_mode && label.is_some() {
            let num = self.fetch_and_inc_num("equation".to_string(), &label);
            args.insert("num", &num);
        }
        args.insert("value", inner);
        ctx.templates.render(
            "math",
            ctx.format.template_prefix(),
            TemplateType::Builtin,
            &args,
            buf,
        )
    }
}

impl RenderElement<Inline> for GenericRenderer {
    fn render(&mut self, elem: &Inline, ctx: &RenderContext, mut buf: impl Write) -> Result<()> {
        match elem {
            Inline::Text(s) => {
                let _ = buf.write(s.as_bytes())?;
                Ok(())
            }
            Inline::Styled(inner, style) => match style {
                Style::Emphasis => render_value_template(
                    "emphasis",
                    TemplateType::Builtin,
                    &self.render_inner(inner, ctx)?,
                    ctx,
                    buf,
                ),
                Style::Strong => render_value_template(
                    "strong",
                    TemplateType::Builtin,
                    &self.render_inner(inner, ctx)?,
                    ctx,
                    buf,
                ),
                Style::Strikethrough => render_value_template(
                    "strikethrough",
                    TemplateType::Builtin,
                    &self.render_inner(inner, ctx)?,
                    ctx,
                    buf,
                ),
                Style::Underline => render_value_template(
                    "underline",
                    TemplateType::Builtin,
                    &self.render_inner(inner, ctx)?,
                    ctx,
                    buf,
                ),
            },
            Inline::CodeBlock(CodeBlock {
                label,
                source,
                attributes,
                ..
            }) => {
                let id = get_id();

                let code_rendered = source.to_string(ctx.doc.meta.code_solutions)?;

                let highlighted = syntect::html::highlighted_html_for_string(
                    &code_rendered,
                    ctx.syntax_set,
                    ctx.syntax_set.find_syntax_by_extension("py").unwrap(),
                    ctx.theme,
                )?;

                let num = self.fetch_and_inc_num("code".to_string(), &label);

                let mut args = Context::default();
                args.insert("label", label);
                args.insert("interactive", &ctx.doc.meta.interactive);
                args.insert("cell_outputs", &ctx.doc.meta.cell_outputs);
                args.insert("editable", &ctx.doc.meta.editable);
                args.insert("source", &code_rendered);
                args.insert("highlighted", &highlighted);
                args.insert("id", &id);
                args.insert("attr", &attributes);
                args.insert("meta", &source.meta);
                args.insert("num", &num);
                // args.insert("outputs", &self.render_inner(outputs, ctx)?);
                // args.insert("outputs", outputs);

                Ok(ctx.templates.render(
                    "cell",
                    ctx.format.template_prefix(),
                    TemplateType::Builtin,
                    &args,
                    buf,
                )?)
            }
            Inline::Code(s) => {
                render_value_template("inline_code", TemplateType::Builtin, s, ctx, buf)
            }
            Inline::SoftBreak => {
                render_basic_template("soft_break", TemplateType::Builtin, ctx, buf)
            }
            Inline::HardBreak => {
                render_basic_template("hard_break", TemplateType::Builtin, ctx, buf)
            }
            Inline::Rule => {
                render_basic_template("horizontal_rule", TemplateType::Builtin, ctx, buf)
            }
            Inline::Image(_tp, url, alt, inner) => {
                let inner = self.render_inner(inner, ctx)?;
                render_image(url, alt, &inner, ctx, buf)
            }
            Inline::Link(_tp, url, alt, inner) => {
                let inner = self.render_inner(inner, ctx)?;
                render_link(url, alt, &inner, ctx, buf)
            }
            Inline::Html(s) => write_bytes(s, buf),
            Inline::Math(Math {
                source,
                display_block,
                label,
                ..
            }) => self.render_math(*display_block, source, label, ctx, buf),
            Inline::Command(command) => self.render_command_template(ctx, command, buf),
        }
    }
}

impl RenderElement<CodeOutput> for GenericRenderer {
    fn render(
        &mut self,
        elem: &CodeOutput,
        ctx: &RenderContext,
        mut buf: impl Write,
    ) -> Result<()> {
        for output in &elem.values {
            match output {
                Outval::Text(text) => {
                    render_value_template(
                        "output_text",
                        TemplateType::Builtin,
                        text,
                        ctx,
                        &mut buf,
                    )?;
                }
                Outval::Image(img) => match img {
                    Image::Png(s) => {
                        render_value_template(
                            "output_img",
                            TemplateType::Builtin,
                            s,
                            ctx,
                            &mut buf,
                        )?;
                    }
                    Image::Svg(s) => {
                        render_value_template(
                            "output_svg",
                            TemplateType::Builtin,
                            s,
                            ctx,
                            &mut buf,
                        )?;
                    }
                },
                Outval::Json(s) => {
                    write_bytes(&serde_json::to_string(s)?, &mut buf)?;
                }
                Outval::Html(s) => {
                    write_bytes(s, &mut buf)?;
                }
                Outval::Javascript(s) => {
                    write_bytes(s, &mut buf)?;
                }
                Outval::Error(text) => {
                    render_value_template(
                        "output_error",
                        TemplateType::Builtin,
                        text,
                        ctx,
                        &mut buf,
                    )?;
                }
            }
        }
        Ok(())
    }
}

impl RenderElement<Block> for GenericRenderer {
    fn render(&mut self, elem: &Block, ctx: &RenderContext, buf: impl Write) -> Result<()> {
        match elem {
            Block::Heading { lvl, inner, .. } => {
                let mut args = Context::default();
                // println!("{}", );
                args.insert("level", &lvl);
                args.insert("inner", &self.render_inner(inner, ctx)?);
                Ok(ctx.templates.render(
                    "header",
                    ctx.format.template_prefix(),
                    TemplateType::Builtin,
                    &args,
                    buf,
                )?)
            }
            Block::Plain(inner) => self.render(inner, ctx, buf),
            Block::Paragraph(inner) | Block::BlockQuote(inner) => render_value_template(
                "paragraph",
                TemplateType::Builtin,
                &self.render_inner(inner, ctx)?,
                ctx,
                buf,
            ),

            Block::List(idx, items) => {
                self.list_level += 1;
                self.current_list_idx.push(*idx);
                let inner = self.render_inner(items, ctx)?;
                // let inner: Result<String> = items.iter().map(|b| self.render(b, ctx)).collect();
                // let inner = inner?;

                match idx {
                    None => render_value_template(
                        "list_unordered",
                        TemplateType::Builtin,
                        &inner,
                        ctx,
                        buf,
                    )?,
                    Some(start) => {
                        let mut args = Context::default();
                        args.insert("start", &start);
                        args.insert("value", &inner);
                        ctx.templates.render(
                            "list_ordered",
                            ctx.format.template_prefix(),
                            TemplateType::Builtin,
                            &args,
                            buf,
                        )?
                    }
                };

                self.list_level -= 1;
                self.current_list_idx.pop();
                Ok(())
            }
            Block::ListItem(inner) => {
                let mut args = Context::default();
                args.insert("lvl", &self.list_level);
                args.insert("idx", &self.current_list_idx.last().unwrap());
                if let Some(i) = self.current_list_idx.last_mut().unwrap().as_mut() {
                    *i += 1;
                }
                args.insert("value", &self.render_inner(inner, ctx)?);
                ctx.templates.render(
                    "list_item",
                    ctx.format.template_prefix(),
                    TemplateType::Builtin,
                    &args,
                    buf,
                )
            }
        }
    }
}

fn render_basic_template(
    name: &str,
    type_: TemplateType,
    ctx: &RenderContext,
    buf: impl Write,
) -> Result<()> {
    ctx.templates.render(
        name,
        ctx.format.template_prefix(),
        type_,
        &Context::default(),
        buf,
    )
}

fn render_value_template(
    name: &str,
    type_: TemplateType,
    value: &str,
    ctx: &RenderContext,
    buf: impl Write,
) -> Result<()> {
    let mut args = Context::default();
    args.insert("value", value);
    ctx.templates
        .render(name, ctx.format.template_prefix(), type_, &args, buf)
}

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn add_args(
    def: &TemplateDefinition,
    args: &mut Context,
    id: &Option<String>,
    num: usize,
    arguments: Vec<RenderedParam>,
) -> Result<()> {
    if let Some(id) = id {
        args.insert("id", &id);
    }
    args.insert("num", &num);

    for (i, p) in arguments.into_iter().enumerate() {
        let key = if let Some(key) = p.key {
            key
        } else {
            def.shortcode.as_ref().unwrap().parameters[i].name.clone()
        };

        args.insert(key, &p.value);
    }
    Ok(())
}

fn render_image(
    url: &str,
    alt: &str,
    inner: &str,
    ctx: &RenderContext,
    buf: impl Write,
) -> Result<()> {
    let mut args = Context::default();
    args.insert("url", url);
    args.insert("alt", alt);
    args.insert("inner", inner);
    ctx.templates.render(
        "image",
        ctx.format.template_prefix(),
        TemplateType::Builtin,
        &args,
        buf,
    )
}

fn render_link(
    url: &str,
    alt: &str,
    inner: &str,
    ctx: &RenderContext,
    buf: impl Write,
) -> Result<()> {
    let mut args = Context::default();
    args.insert("url", url);
    args.insert("alt", alt);
    args.insert("inner", inner);
    ctx.templates.render(
        "link",
        ctx.format.template_prefix(),
        TemplateType::Builtin,
        &args,
        buf,
    )
}
