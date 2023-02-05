use crate::ast::{AEvent, Ast, Block, Inline};
use crate::document::{Document, EventContent};
use pulldown_cmark::html;
use serde::{Deserialize, Serialize};

use crate::renderers::{RenderResult, Renderer};

#[derive(Serialize, Deserialize)]
pub struct HtmlRenderer {
    pub(crate) interactive_cells: bool,
}

#[typetag::serde(name = "renderer_config")]
impl Renderer for HtmlRenderer {
    fn render(&self, doc: &Document<Ast>) -> Document<RenderResult> {
        // let doc = doc.to_events();
        // let dd = doc.to_events();
        //
        // let mut output = String::new();
        // html::push_html(&mut output, dd);
        Document {
            content: doc.content.0.clone().to_html(),
            metadata: doc.metadata.clone(),
            variables: doc.variables.clone(),
        }
    }
}

pub trait ToHtml {
    fn to_html(self) -> String;
}

impl ToHtml for Vec<Inline> {
    fn to_html(self) -> String {
        self.into_iter().map(|i| i.to_html()).collect()
    }
}

impl ToHtml for Vec<Block> {
    fn to_html(self) -> String {
        self.into_iter().map(|b| b.to_html()).collect()
    }
}

impl ToHtml for Inline {
    fn to_html(self) -> String {
        match self {
            Inline::Text(s) => s,
            Inline::Emphasis(inner) | Inline::Strong(inner) | Inline::Strikethrough(inner) => {
                inner.to_html()
            }
            Inline::Code(s) => s,
            Inline::SoftBreak => String::default(),
            Inline::HardBreak => String::default(),
            Inline::Rule => "<hr>".to_string(),
            Inline::Image(tp, url, title) => format!(r#"<img src="{url}">{title}</img>"#),
            Inline::Link(tp, url, title) => format!(r#"<a href="{url}">{title}</a>"#),
            Inline::Html(s) => s,
        }
    }
}

impl ToHtml for Block {
    fn to_html(self) -> String {
        match self {
            Block::Heading {
                lvl,
                id,
                classes,
                inner,
            } => {
                format!("<{lvl}>{}</{lvl}>", inner.to_html())
            }
            Block::Plain(inner) => inner.to_html(),
            Block::Paragraph(inner) | Block::BlockQuote(inner) => inner.to_html(),
            Block::CodeBlock {
                source,
                reference,
                attr,
                outputs,
            } => {
                format!("<pre><code>{source}</code></pre>")
            }
            Block::List(idx, items) => {
                let inner: String = items.into_iter().map(|b| b.to_html()).collect();
                match idx {
                    None => format!("<ul>{inner}</ul>"),
                    Some(start) => format!(r#"<ol start="{start}">{inner}</ol>"#),
                }
            }
            Block::ListItem(inner) => format!("<li>{}</li>", inner.to_html()),
        }
    }
}
