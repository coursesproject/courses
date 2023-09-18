use crate::raw::{CodeAttr, RawDocument, Reference};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammars/raw_doc.pest"]
pub struct RawDocParser;

use crate::code_ast::parse_code_string;
use crate::common::PosInfo;
use crate::raw::{Element, ElementInfo, Parameter, Special, Value};
use pest::iterators::Pairs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("code cell parsing error")]
    CodeError(#[from] Box<pest::error::Error<crate::code_ast::Rule>>),
    #[error("document parsing error")]
    DocError(#[from] Box<pest::error::Error<Rule>>),
}

impl RawDocument {
    fn parse_doc(&mut self, mut pairs: Pairs<Rule>) -> Result<(), ParserError> {
        let mut elems = pairs.next().expect("no root item").into_inner();

        if let Some(p) = elems.next() {
            match p.as_rule() {
                Rule::meta => self.parse_meta(p),
                _ => {
                    let el = self.parse_element(p);
                    self.src.push(el?)
                }
            }
        }

        let elems = self.parse_elements(elems)?;
        self.src.extend(elems);

        Ok(())
    }

    fn parse_elements(&mut self, pairs: Pairs<Rule>) -> Result<Vec<ElementInfo>, ParserError> {
        pairs.map(|p| self.parse_element(p.clone())).collect()
    }

    fn parse_element(&mut self, pair: Pair<Rule>) -> Result<ElementInfo, ParserError> {
        let span = PosInfo::from(pair.as_span());

        let element = match pair.as_rule() {
            Rule::command => self.parse_command(pair)?,
            Rule::math_block => self.parse_math(pair),
            Rule::code_def => self.parse_code(pair)?,
            Rule::verbatim => self.parse_verbatim(pair),
            Rule::src | Rule::string | Rule::body => self.parse_src(pair),
            _ => unreachable!(),
        };

        Ok(ElementInfo { element, pos: span })
    }

    fn parse_src(&mut self, pair: Pair<Rule>) -> Element {
        let value = pair.as_str();
        Element::Markdown(value.into())
    }

    fn parse_command(&mut self, pair: Pair<Rule>) -> Result<Element, ParserError> {
        let mut inner = pair.into_inner();
        let name = inner
            .next()
            .expect("empty command")
            .into_inner()
            .next()
            .unwrap()
            .as_str();

        let mut parameters = vec![];
        let mut body = None;
        let mut label = None;

        for elem in inner {
            match elem.as_rule() {
                Rule::parameters => parameters = self.parse_parameters(elem.into_inner())?,
                Rule::body_def => body = Some(self.parse_elements(elem.into_inner())?),
                Rule::label => label = Some(elem.into_inner().as_str().to_string()),
                _ => unreachable!(),
            }
        }

        if let Some(label) = label.clone() {
            self.references.insert(
                label,
                Reference::Command(name.to_string(), parameters.clone()),
            );
        }

        Ok(Element::Special(
            label,
            Special::Command {
                function: name.into(),
                parameters,
                body,
            },
        ))
    }

    fn parse_parameters(&mut self, pairs: Pairs<Rule>) -> Result<Vec<Parameter>, ParserError> {
        pairs
            .into_iter()
            .map(|elem| {
                if let Rule::param = elem.as_rule() {
                    self.parse_param(elem)
                } else {
                    unreachable!()
                }
            })
            .collect()
    }

    fn parse_param(&mut self, pair: Pair<Rule>) -> Result<Parameter, ParserError> {
        let span = PosInfo::from(pair.as_span());
        let mut pairs = pair.into_inner();
        let first = pairs.next().expect("empty param");

        Ok(if let Rule::key = first.as_rule() {
            let value = pairs.next().expect("no value");
            Parameter::with_key(first.as_str(), self.parse_value(value)?, span)
        } else {
            Parameter::with_value(self.parse_value(first)?, span)
        })
    }

    fn parse_value(&mut self, pair: Pair<Rule>) -> Result<Value, ParserError> {
        Ok(match pair.as_rule() {
            Rule::basic_val | Rule::string => Value::String(pair.as_str().into()),
            Rule::md_val => Value::Content(self.parse_elements(pair.into_inner())?),
            Rule::flag => Value::Flag(pair.as_str().into()),
            _ => unreachable!(),
        })
    }

    fn block_parser(&mut self, pair: Pair<Rule>) -> (String, String, Option<String>) {
        let mut inner = pair.into_inner();
        let lvl = inner.next().expect("missing code_lvl").as_str().to_string();
        let src = inner.next().expect("missing code_src").as_str().to_string();
        let id = inner.next().map(|val| val.as_str().to_string());
        (lvl, src, id)
    }

    fn parse_math(&mut self, pair: Pair<Rule>) -> Element {
        let (lvl, src, label) = self.block_parser(pair);

        if let Some(label) = label.clone() {
            self.references
                .insert(label, Reference::Math(src.to_string()));
        }

        Element::Special(
            label,
            Special::Math {
                inner: src,
                is_block: lvl.len() != 1,
            },
        )
    }

    fn parse_code_attributes(&mut self, pairs: Pairs<Rule>) -> Vec<CodeAttr> {
        pairs
            .into_iter()
            .map(|elem| {
                if let Rule::code_param = elem.as_rule() {
                    self.parse_code_attribute(elem)
                } else {
                    unreachable!()
                }
            })
            .collect()
    }

    fn parse_code_attribute(&mut self, pair: Pair<Rule>) -> CodeAttr {
        let mut pairs = pair.into_inner();
        let first = pairs.next().expect("empty param");

        if let Rule::key = first.as_rule() {
            let value = pairs.next().expect("no value");
            CodeAttr {
                key: Some(first.as_str().to_string()),
                value: value.as_str().to_string(),
            }
        } else {
            CodeAttr {
                key: None,
                value: first.as_str().to_string(),
            }
        }
    }

    fn parse_code(&mut self, pair: Pair<Rule>) -> Result<Element, ParserError> {
        let mut inner = pair.into_inner();
        let lvl = inner.next().expect("missing code_lvl").as_str().to_string();

        let maybe_param = inner.next().expect("missing code_src");
        let (src_pair, params) = if let Rule::code_params = maybe_param.as_rule() {
            let params = self.parse_code_attributes(maybe_param.into_inner());
            (inner.next().expect("missing code_src"), Some(params))
        } else {
            (maybe_param, None)
        };

        let src = src_pair.as_str().to_string();

        let id = inner.next().map(|val| val.as_str().to_string());

        if let Some(label) = id.clone() {
            self.references
                .insert(label, Reference::Code(src.to_string()));
        }

        Ok(Element::Special(
            id,
            if lvl.len() == 1 {
                Special::CodeInline { inner: src }
            } else {
                let content = parse_code_string(src.trim())?;

                Special::CodeBlock {
                    lvl: lvl.len(),
                    inner: content,
                    params: params.unwrap_or_default(),
                }
            },
        ))
    }

    fn parse_verbatim(&mut self, pair: Pair<Rule>) -> Element {
        let value = pair.as_str();
        Element::Special(
            None,
            Special::Verbatim {
                inner: value.into(),
            },
        )
    }

    fn parse_meta(&mut self, pair: Pair<Rule>) {
        self.meta = Some(pair.as_str().to_string());
    }
}

pub fn parse_to_doc(input: &str) -> Result<RawDocument, ParserError> {
    let mut doc = RawDocument::default();
    doc.parse_doc(RawDocParser::parse(Rule::top, input).map_err(Box::new)?)?;

    Ok(doc)
}

#[cfg(test)]
mod tests {
    use crate::code_ast::types::{CodeBlock, CodeContent};
    use crate::common::PosInfo;
    use crate::raw::{
        parse_to_doc, CodeAttr, Element, ElementInfo, Parameter, RawDocument, Reference, Special,
        Value,
    };
    use std::collections::HashMap;

    macro_rules! doc_tests {
        ($prefix:ident $($name:ident: $value:expr,)*) => {
        $(
            paste::item!{
            #[test]
            fn [<$prefix _ $name>]() {
                let (input, expected) = $value;
                let doc = RawDocument { src: expected, meta: None, references: Default::default() };
                compare(doc, input);
            }
            }
        )*
        }
    }

    #[test]
    fn test_code() {
        let input = r#"```
code
```"#;
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Special(
                    None,
                    Special::CodeBlock {
                        lvl: 3,
                        inner: CodeContent {
                            blocks: vec![CodeBlock::Src("code\n".into())],
                            meta: Default::default(),
                            hash: 7837613302888775477,
                        },
                        params: vec![],
                    },
                ),
                pos: PosInfo::new(input, 0, 12),
            }],
            meta: None,
            references: Default::default(),
        };

        compare(expected, input);
    }

    #[test]
    fn test_code_param() {
        let input = r#"```lang, key=val
code
```"#;
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Special(
                    None,
                    Special::CodeBlock {
                        lvl: 3,
                        inner: CodeContent {
                            blocks: vec![CodeBlock::Src("code\n".into())],
                            meta: Default::default(),
                            hash: 7837613302888775477,
                        },
                        params: vec![
                            CodeAttr {
                                key: None,
                                value: "lang".to_string(),
                            },
                            CodeAttr {
                                key: Some("key".to_string()),
                                value: "val".to_string(),
                            },
                        ],
                    },
                ),
                pos: PosInfo::new(input, 0, 25),
            }],
            meta: None,
            references: Default::default(),
        };

        compare(expected, input);
    }

    #[test]
    fn test_math() {
        let input = "$inline$";
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Special(
                    None,
                    Special::Math {
                        is_block: false,
                        inner: "inline".into(),
                    },
                ),
                pos: PosInfo::new(input, 0, 8),
            }],
            meta: None,
            references: Default::default(),
        };

        compare(expected, input);
    }

    #[test]
    fn test_verbatim() {
        let input = "\\{verbatim\\}";
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Special(
                    None,
                    Special::Verbatim {
                        inner: "verbatim".into(),
                    },
                ),
                pos: PosInfo::new(input, 2, 10),
            }],
            meta: None,
            references: Default::default(),
        };

        compare(expected, input);
    }

    #[test]
    fn test_src() {
        let input = "just some stuff {} xx--^*# fsdf";
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Markdown(input.into()),
                pos: PosInfo::new(input, 0, 31),
            }],
            meta: None,
            references: Default::default(),
        };

        compare(expected, input);
    }

    #[test]
    fn test_refs() {
        let input = "#call|id";
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Special(
                    Some("id".to_string()),
                    Special::Command {
                        function: "call".to_string(),
                        parameters: vec![],
                        body: None,
                    },
                ),
                pos: PosInfo::new(input, 0, 8),
            }],
            meta: None,
            references: HashMap::from([(
                "id".to_string(),
                Reference::Command("call".to_string(), vec![]),
            )]),
        };

        compare(expected, input);
    }

    const CMD_WITH_PARAMS_NO_BODY: &str =
        "#func(basic, \"quoted\", {content}, key=basic, key=\"quoted\", key={content}, :flag)";

    doc_tests! {
        command
        no_params_no_body: ("#func",  vec![
            ElementInfo {
                element: Element::Special(None, Special::Command {
                    function: "func".into(),
                    parameters: vec![],
                    body: None,
                }),
                pos: PosInfo::new("#func", 0, 5),
            }
        ]),
        with_params_no_body: (CMD_WITH_PARAMS_NO_BODY,  vec![
            ElementInfo {
                element: Element::Special(None, Special::Command {
                    function: "func".into(),
                    parameters: vec![
                        Parameter { key: None, value: Value::String("basic".into()), pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 6, 11) },
                        Parameter { key: None, value: Value::String("quoted".into()), pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 13, 21) },
                        Parameter { key: None, value: Value::Content(vec![
                            ElementInfo {
                                element: Element::Markdown("content".into()),
                                pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 24, 31)
                            }
                        ]), pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 23, 32) },
                        Parameter { key: Some("key".into()), value: Value::String("basic".into()), pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 34, 43) },
                        Parameter { key: Some("key".into()), value: Value::String("quoted".into()), pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 45, 57) },
                        Parameter { key: Some("key".into()), value: Value::Content(vec![
                            ElementInfo {
                                element: Element::Markdown("content".into()),
                                pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 64, 71)
                            }
                        ]), pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 59, 72) },
                        Parameter { key: None, value: Value::Flag("flag".into()), pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 74, 79) }
                    ],
                    body: None,
                }),
                pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 0, 80),
            }
        ]),
        with_params_with_body: ("#func(c){x}", vec![
            ElementInfo {
                element: Element::Special(None, Special::Command {
                    function: "func".into(),
                    parameters: vec![
                        Parameter { key: None, value: Value::String("c".to_string()), pos: PosInfo::new("#func(c){x}", 6, 7)}
                    ],
                    body: Some(vec![
                        ElementInfo {
                            element: Element::Markdown("x".into()),
                            pos: PosInfo::new("#func(c){x}", 9, 10)
                        }
                    ])
                }),
                pos: PosInfo::new("#func(c){x}", 0, 11),
            }
        ]),
        no_params_with_body: ("#func{x}", vec![
            ElementInfo {
                element: Element::Special(None, Special::Command {
                    function: "func".into(),
                    parameters: vec![],
                    body: Some(vec![
                        ElementInfo {
                            element: Element::Markdown("x".into()),
                            pos: PosInfo::new("#func{x}", 6, 7)
                        }
                    ])
                }),
                pos: PosInfo::new("#func{x}", 0, 8),
            }
        ]),
        body_nested: ("#func1{#func2}", vec![
            ElementInfo {
                element: Element::Special(None, Special::Command {
                    function: "func1".into(),
                    parameters: vec![],
                    body: Some(vec![ElementInfo {
                            element: Element::Special(None, Special::Command{
                                function: "func2".into(),
                                parameters: vec![],
                                body: None,
                            }),
                            pos: PosInfo::new("#func1{#func2}", 7, 13),
                        }
                    ])
                }),
                pos: PosInfo::new("#func1{#func2}", 0, 14),
            }

        ]),
    }

    fn compare(expected: RawDocument, input: &str) {
        let doc = parse_to_doc(input).expect("Parse error");

        assert_eq!(expected, doc);
    }
}
