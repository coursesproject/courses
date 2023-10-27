use crate::code_ast::types::CodeElem;
use crate::raw::{ArgumentVal, Child, ComposedMarkdown, Special};
use cdoc_base::module::Module;
use cdoc_base::node::{Attribute, ChildType, Compound, Node, Script};
use lazy_static::lazy_static;
use pulldown_cmark::{Event, HeadingLevel, Parser as MdParser, Tag};
use regex::Regex;
use std::str::FromStr;

fn to_vec_element<T>(vec: Vec<T>) -> Vec<Node>
where
    Node: From<T>,
{
    vec.into_iter().map(Node::from).collect()
}

impl From<CodeElem> for Node {
    fn from(value: CodeElem) -> Self {
        match value {
            CodeElem::Solution(s) => {
                let mut children = vec![Node::Compound(Compound::new_with_children(
                    "solution",
                    None,
                    vec![Node::Plain(s.solution.to_string())],
                ))];
                if let Some(placeholder) = s.placeholder {
                    children.push(Node::Compound(Compound::new_with_children(
                        "placeholder",
                        None,
                        vec![Node::Plain(placeholder.to_string())],
                    )))
                }
                Node::Compound(Compound::new_with_children(
                    "solution_block",
                    None,
                    children,
                ))
            }
            CodeElem::Src(s) => Node::Plain(s),
        }
    }
}

impl From<Child> for Node {
    fn from(value: Child) -> Self {
        let mut attributes = vec![];

        match value.elem {
            Special::Math { inner, is_block } => {
                if is_block {
                    Node::Compound(Compound::new(
                        "math_block",
                        Some(value.label.as_str()),
                        attributes,
                        vec![Node::Plain(inner.to_string())],
                    ))
                } else {
                    Node::Compound(Compound::new(
                        "math",
                        Some(value.label.as_str()),
                        attributes,
                        vec![Node::Plain(inner.to_string())],
                    ))
                }
            }
            Special::Script { id, src, children } => {
                let elements = children
                    .into_iter()
                    .map(|c| ComposedMarkdown::from(c).into())
                    .collect();
                Node::Script(Script {
                    id,
                    src: src.to_string(),
                    elements,
                })
            }
            Special::Command {
                function,
                parameters,
                body,
            } => {
                let mut children = vec![];
                for (i, parameter) in parameters.into_iter().enumerate() {
                    match parameter.value {
                        ArgumentVal::Flag(f) => {
                            attributes.push((f.to_string(), Attribute::Flag));
                        }
                        ArgumentVal::Content(c) => {
                            let composed = ComposedMarkdown::from(c);
                            let mut elems: Vec<Node> = composed.into();

                            let el = if elems.len() == 1 {
                                if let Node::Compound(n) = elems.remove(0) {
                                    if n.type_id == "paragraph" {
                                        n.children
                                    } else {
                                        vec![Node::Compound(n)]
                                    }
                                } else {
                                    elems
                                }
                            } else {
                                elems
                            };

                            attributes.push((
                                parameter.key.unwrap().to_string(),
                                Attribute::Compound(el),
                            ));
                            // children.push(Element::Node(Node::new(
                            //     format!("parameter:{}", parameter.key.as_ref().unwrap()),
                            //     [(
                            //         "name".to_string(),
                            //         Attribute::String(parameter.key.unwrap().to_string()),
                            //     )],
                            //     composed.into(),
                            // )));
                        }
                        ArgumentVal::String(s) => attributes.push((
                            parameter
                                .key
                                .map(|k| k.to_string())
                                .unwrap_or(format!("pos_{}", i)),
                            Attribute::String(s.to_string()),
                        )),
                        ArgumentVal::Int(i) => attributes.push((
                            parameter
                                .key
                                .map(|k| k.to_string())
                                .unwrap_or(format!("pos_{}", i)),
                            Attribute::Int(i),
                        )),
                        ArgumentVal::Float(f) => attributes.push((
                            parameter
                                .key
                                .map(|k| k.to_string())
                                .unwrap_or(format!("pos_{}", i)),
                            Attribute::Float(f),
                        )),
                    }
                }
                if let Some(body) = body {
                    let composed = ComposedMarkdown::from(body);
                    let mut elems: Vec<Node> = composed.into();

                    let el = if elems.len() == 1 {
                        // TODO: Truly horrific
                        if let Node::Compound(n) = elems.remove(0) {
                            if n.type_id == "paragraph" {
                                n.children
                            } else {
                                vec![Node::Compound(n)]
                            }
                        } else {
                            elems
                        }
                    } else {
                        elems
                    };

                    children.extend(el);
                }

                Node::Compound(Compound::new(
                    function.to_string(),
                    Some(value.label.as_str()),
                    attributes,
                    children,
                ))
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
                Node::Compound(Compound::new(
                    "code_block",
                    Some(value.label.as_str()),
                    attributes,
                    inner.blocks.into_iter().map(|e| e.into()).collect(),
                ))
            }
            Special::CodeInline { inner } => Node::Compound(Compound::new_with_children(
                "code",
                Some(value.label.as_str()),
                vec![Node::Plain(inner.to_string())],
            )),
            Special::Verbatim { inner } => Node::Plain(inner.to_string()),
        }
    }
}

lazy_static! {
    static ref r: Regex = Regex::new(r"elem-([0-9]+)").expect("invalid regex expression");
}

impl From<ComposedMarkdown> for Vec<Node> {
    fn from(value: ComposedMarkdown) -> Self {
        let parser: MdParser = MdParser::new(&value.src);
        let mut nodes = vec![Vec::new()];

        for event in parser {
            match event {
                Event::Start(_) => nodes.push(Vec::new()),
                Event::End(t) => {
                    let children = nodes.pop().expect("Missing children");
                    let current_node = nodes.last_mut().unwrap();
                    match t {
                        Tag::Paragraph => current_node.push(Node::Compound(
                            Compound::new_with_children("paragraph", None, children),
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
                            current_node.push(Node::Compound(Compound::new(
                                "heading", None, attributes, children,
                            )))
                        }
                        Tag::List(idx) => current_node.push(Node::Compound(Compound::new(
                            "list",
                            None,
                            idx.map(|idx| {
                                vec![("start_idx".to_string(), Attribute::Int(idx as i64))]
                            })
                            .unwrap_or_default(),
                            children,
                        ))),
                        Tag::Item => current_node.push(Node::Compound(
                            Compound::new_with_children("list_item", None, children),
                        )),
                        Tag::Emphasis => current_node.push(Node::Compound(Compound::new(
                            "emphasis",
                            None,
                            [],
                            children,
                        ))),
                        Tag::Strong => current_node.push(Node::Compound(Compound::new(
                            "strong",
                            None,
                            [],
                            children,
                        ))),
                        Tag::Strikethrough => current_node.push(Node::Compound(Compound::new(
                            "strikethrough",
                            None,
                            [],
                            children,
                        ))),
                        Tag::Link(_, url, alt) => current_node.push(Node::Compound(Compound::new(
                            "link",
                            None,
                            [
                                ("url".to_string(), Attribute::String(url.to_string())),
                                ("alt".to_string(), Attribute::String(alt.to_string())),
                            ],
                            children,
                        ))),
                        Tag::Image(_, url, alt) => {
                            current_node.push(Node::Compound(Compound::new(
                                "image",
                                None,
                                [
                                    ("url".to_string(), Attribute::String(url.to_string())),
                                    ("alt".to_string(), Attribute::String(alt.to_string())),
                                ],
                                children,
                            )))
                        }
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
                        nodes.last_mut().unwrap().push(Node::Plain(src.to_string()));
                    }
                }
                Event::Text(text) => nodes
                    .last_mut()
                    .unwrap()
                    .push(Node::Plain(text.to_string())),
                Event::SoftBreak => nodes
                    .last_mut()
                    .unwrap()
                    .push(Node::Compound(Compound::new_empty("soft_break", None))),
                Event::HardBreak => nodes
                    .last_mut()
                    .unwrap()
                    .push(Node::Compound(Compound::new_empty("hard_break", None))),
                Event::Rule => nodes
                    .last_mut()
                    .unwrap()
                    .push(Node::Compound(Compound::new_empty("rule", None))),
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
