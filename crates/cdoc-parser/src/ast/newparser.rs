use crate::code_ast::types::CodeElem;
use crate::raw::{Child, ComposedMarkdown, Special, Value};
use cdoc_base::module::Module;
use cdoc_base::node::{Attribute, ChildType, Element, Node};
use pulldown_cmark::{Event, HeadingLevel, Parser as MdParser, Tag};
use regex::Regex;
use std::str::FromStr;

fn to_vec_element<T>(vec: Vec<T>) -> Vec<Element>
where
    Element: From<T>,
{
    vec.into_iter().map(Element::from).collect()
}

impl From<CodeElem> for Element {
    fn from(value: CodeElem) -> Self {
        match value {
            CodeElem::Solution(s) => {
                let mut children = vec![Element::Node(Node::new_with_children(
                    "solution",
                    vec![Element::Plain(s.solution.to_string())],
                ))];
                if let Some(placeholder) = s.placeholder {
                    children.push(Element::Node(Node::new_with_children(
                        "placeholder",
                        vec![Element::Plain(placeholder.to_string())],
                    )))
                }
                Element::Node(Node::new_with_children("solution_block", children))
            }
            CodeElem::Src(s) => Element::Plain(s),
        }
    }
}

impl From<Child> for Element {
    fn from(value: Child) -> Self {
        let mut attributes = vec![];
        if let Some(label) = value.label {
            attributes.push(("label".to_string(), Attribute::String(label.to_string())));
        }
        match value.elem {
            Special::Math { inner, is_block } => {
                if is_block {
                    Element::Node(Node::new(
                        "math_block",
                        attributes,
                        vec![Element::Plain(inner.to_string())],
                    ))
                } else {
                    Element::Node(Node::new(
                        "math",
                        attributes,
                        vec![Element::Plain(inner.to_string())],
                    ))
                }
            }
            Special::Command {
                function,
                parameters,
                body,
            } => {
                let mut children = vec![];
                for parameter in parameters {
                    match parameter.value {
                        Value::Flag(f) => {
                            attributes.push((f.to_string(), Attribute::Flag));
                        }
                        Value::Content(c) => {
                            let composed = ComposedMarkdown::from(c);
                            children.push(Element::Node(Node::new(
                                format!("parameter:{}", parameter.key.as_ref().unwrap()),
                                [(
                                    "name".to_string(),
                                    Attribute::String(parameter.key.unwrap().to_string()),
                                )],
                                composed.into(),
                            )));
                        }
                        Value::String(s) => attributes.push((
                            parameter
                                .key
                                .map(|k| k.to_string())
                                .unwrap_or("positional".to_string()),
                            Attribute::String(s.to_string()),
                        )),
                    }
                }
                if let Some(body) = body {
                    let composed = ComposedMarkdown::from(body);
                    let elems: Vec<Element> = composed.into();
                    children.extend(elems);
                }

                Element::Node(Node::new(function.to_string(), attributes, children))
            }
            Special::CodeBlock {
                lvl,
                inner,
                attributes: attr,
            } => {
                if let Some(lang) = attr.get(0) {
                    attributes.push(("language".to_string(), Attribute::String(lang.to_string())));
                }
                if let Some(_) = attr.get(1) {
                    attributes.push(("is_cell".to_string(), Attribute::Flag));
                }
                inner.meta.iter().for_each(|(k, v)| {
                    attributes.push((k.to_string(), Attribute::String(v.to_string())))
                });
                Element::Node(Node::new(
                    "code_block",
                    attributes,
                    to_vec_element(inner.blocks),
                ))
            }
            Special::CodeInline { inner } => Element::Node(Node::new_with_children(
                "code",
                vec![Element::Plain(inner.to_string())],
            )),
            Special::Verbatim { inner } => Element::Plain(inner.to_string()),
        }
    }
}

impl From<ComposedMarkdown> for Vec<Element> {
    fn from(value: ComposedMarkdown) -> Self {
        let parser: MdParser = MdParser::new(&value.src);
        let r = Regex::new(r"elem-([0-9]+)").expect("invalid regex expression");
        let mut nodes = vec![Vec::new()];

        for event in parser {
            match event {
                Event::Start(t) => nodes.push(Vec::new()),
                Event::End(t) => {
                    let children = nodes.pop().expect("Missing children");
                    let current_node = nodes.last_mut().unwrap();
                    match t {
                        Tag::Paragraph => current_node.push(Element::Node(
                            Node::new_with_children("paragraph", children),
                        )),
                        Tag::Heading(level, label, _) => {
                            let mut attributes = vec![];
                            if let Some(label) = label {
                                attributes.push((
                                    "label".to_string(),
                                    Attribute::String(label.to_string()),
                                ));
                            }
                            attributes
                                .push(("level".to_string(), Attribute::Int(heading_to_lvl(level))));
                            current_node
                                .push(Element::Node(Node::new("heading", attributes, children)))
                        }
                        Tag::List(_) => {}
                        Tag::Item => {}
                        Tag::Emphasis => current_node.push(Element::Node(Node::new(
                            "styled",
                            [(
                                "style".to_string(),
                                Attribute::String("emphasis".to_string()),
                            )],
                            children,
                        ))),
                        Tag::Strong => current_node.push(Element::Node(Node::new(
                            "styled",
                            [("style".to_string(), Attribute::String("strong".to_string()))],
                            children,
                        ))),
                        Tag::Strikethrough => current_node.push(Element::Node(Node::new(
                            "styled",
                            [(
                                "style".to_string(),
                                Attribute::String("strikethrough".to_string()),
                            )],
                            children,
                        ))),
                        Tag::Link(_, url, alt) => current_node.push(Element::Node(Node::new(
                            "link",
                            [
                                ("url".to_string(), Attribute::String(url.to_string())),
                                ("alt".to_string(), Attribute::String(alt.to_string())),
                            ],
                            children,
                        ))),
                        Tag::Image(_, url, alt) => current_node.push(Element::Node(Node::new(
                            "image",
                            [
                                ("url".to_string(), Attribute::String(url.to_string())),
                                ("alt".to_string(), Attribute::String(alt.to_string())),
                            ],
                            children,
                        ))),
                        _ => {} // Missing on purpose/so far
                    }
                }
                Event::Html(src) => {
                    let is_insert = r.captures(src.as_ref()).and_then(|c| c.get(1));

                    if let Some(match_) = is_insert {
                        let idx = usize::from_str(match_.as_str()).unwrap();
                        let elem = value.children[idx].clone();
                        nodes.last_mut().unwrap().push(elem.into());
                    } else {
                        nodes
                            .last_mut()
                            .unwrap()
                            .push(Element::Node(Node::new_with_children(
                                "HTML",
                                vec![Element::Plain(src.to_string())],
                            )));
                    }
                }
                Event::Text(text) => nodes
                    .last_mut()
                    .unwrap()
                    .push(Element::Plain(text.to_string())),
                Event::SoftBreak => nodes
                    .last_mut()
                    .unwrap()
                    .push(Element::Node(Node::new_empty("soft_break"))),
                Event::HardBreak => nodes
                    .last_mut()
                    .unwrap()
                    .push(Element::Node(Node::new_empty("hard_break"))),
                Event::Rule => nodes
                    .last_mut()
                    .unwrap()
                    .push(Element::Node(Node::new_empty("rule"))),
                _ => {} // Missing on purpose
            }
        }

        nodes.remove(0)
    }
}

fn heading_to_lvl(value: HeadingLevel) -> i64 {
    match value {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}
