use crate::parsers::shortcodes::{parse_shortcode, ShortCodeParser};
use anyhow::Context;
use pulldown_cmark::{CowStr, Event};
use std::fmt::{Display, Formatter};
use std::path::Path;
use tera::Tera;

pub struct ShortCodeExtender<'a, I> {
    iter: I,
    extra: Vec<Event<'a>>,
    tera: &'a Tera,
}

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

impl<'a, I> ShortCodeExtender<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    pub fn from_iter(iter: I, tera: &'a Tera) -> anyhow::Result<Self> {
        Ok(ShortCodeExtender {
            iter,
            extra: Vec::new(),
            tera,
        })
    }
}

fn find_shortcode(input: &str) -> Option<(usize, usize)> {
    let start = input.find("{{")?;
    let end = start + 2 + input[start..].find("}}")?;
    Some((start, end))
}

impl<'a, I> ShortCodeExtender<'a, I> {
    fn render_template(&self, shortcode: &str) -> tera::Result<String> {
        let code =
            parse_shortcode(shortcode).ok_or(tera::Error::msg("Invalid shortcode format"))?;
        let mut context = tera::Context::new();
        let name = format!("{}.tera.html", code.name);
        for (k, v) in code.parameters {
            context.insert(k, &v);
        }
        self.tera.render(&name, &context)
    }
}

impl<'a, I> Iterator for ShortCodeExtender<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.extra.is_empty() {
            self.iter.next().map(|event| match event {
                Event::Text(txt) => match find_shortcode(txt.as_ref()) {
                    None => Event::Text(txt),
                    Some((start, end)) => {
                        let pre = &txt[..start];
                        let post = &txt[end..];
                        let tmp_name = (&txt[start..end]).trim();

                        let res = match self.render_template(tmp_name) {
                            Ok(res) => res,
                            Err(e) => e.to_string(),
                        };

                        let html = Event::Html(CowStr::Boxed(res.into_boxed_str()));

                        self.extra.push(html);
                        self.extra.push(Event::Text(CowStr::Boxed(
                            post.to_string().into_boxed_str(),
                        )));
                        Event::Text(CowStr::Boxed(pre.to_string().into_boxed_str()))
                    }
                },
                _ => event,
            })
        } else {
            Some(self.extra.pop().unwrap())
        }
    }
}
