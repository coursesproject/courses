use crate::raw::{RawDocument, Reference};
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammars/raw_doc.pest"]
pub struct RawDocParser;

use crate::common::PosInfo;
use crate::raw::{Element, ElementInfo, Extern, Parameter, Value};
use pest::iterators::Pairs;

impl RawDocument {
    fn parse_doc(&mut self, mut pairs: Pairs<Rule>) -> Result<(), Error<Rule>> {
        let mut elems = pairs.next().expect("no root item").into_inner();

        if let Some(p) = elems.next() {
            match p.as_rule() {
                Rule::meta => self.parse_meta(p),
                _ => {
                    let el = self.parse_element(p);
                    self.src.push(el)
                }
            }
        }

        let elems = self.parse_elements(elems);
        self.src.extend(elems);

        Ok(())
    }

    fn parse_elements(&mut self, pairs: Pairs<Rule>) -> Vec<ElementInfo> {
        pairs.map(|p| self.parse_element(p.clone())).collect()
    }

    fn parse_element(&mut self, pair: Pair<Rule>) -> ElementInfo {
        let span = PosInfo::from(pair.as_span());

        let element = match pair.as_rule() {
            Rule::command => Element::Extern(self.parse_command(pair)),
            Rule::math_block => Element::Extern(self.parse_math(pair)),
            Rule::code_def => Element::Extern(self.parse_code(pair)),
            Rule::verbatim => Element::Extern(self.parse_verbatim(pair)),
            Rule::src | Rule::string | Rule::body => self.parse_src(pair),
            _ => unreachable!(),
        };

        ElementInfo { element, pos: span }
    }

    fn parse_src(&mut self, pair: Pair<Rule>) -> Element {
        let value = pair.as_str();
        Element::Markdown(value.into())
    }

    fn parse_command(&mut self, pair: Pair<Rule>) -> Extern {
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
        let mut id = None;

        while let Some(elem) = inner.next() {
            match elem.as_rule() {
                Rule::parameters => parameters = self.parse_parameters(elem.into_inner()),
                Rule::body_def => body = Some(self.parse_elements(elem.into_inner())),
                Rule::id => id = Some(elem.into_inner().as_str().into()),
                _ => unreachable!(),
            }
        }

        if let Some(id) = id.clone() {
            self.references
                .insert(id, Reference::Command(name.to_string(), parameters.clone()));
        }

        Extern::Command {
            function: name.into(),
            id,
            parameters,
            body,
        }
    }

    fn parse_parameters(&mut self, pairs: Pairs<Rule>) -> Vec<Parameter> {
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

    fn parse_param(&mut self, pair: Pair<Rule>) -> Parameter {
        let span = PosInfo::from(pair.as_span());
        let mut pairs = pair.into_inner();
        let first = pairs.next().expect("empty param");

        if let Rule::key = first.as_rule() {
            let value = pairs.next().expect("no value");
            Parameter::with_key(first.as_str(), self.parse_value(value), span)
        } else {
            Parameter::with_value(self.parse_value(first), span)
        }
    }

    fn parse_value(&mut self, pair: Pair<Rule>) -> Value {
        match pair.as_rule() {
            Rule::basic_val | Rule::string => Value::String(pair.as_str().into()),
            Rule::md_val => Value::Content(self.parse_elements(pair.into_inner())),
            Rule::flag => Value::Flag(pair.as_str().into()),
            _ => unreachable!(),
        }
    }

    fn block_parser(&mut self, pair: Pair<Rule>) -> (String, String) {
        let mut inner = pair.into_inner();
        let lvl = inner.next().expect("missing code_lvl").as_str().to_string();
        let src = inner.next().expect("missing code_src").as_str().to_string();
        (lvl, src)
    }

    fn parse_math(&mut self, pair: Pair<Rule>) -> Extern {
        let (lvl, src) = self.block_parser(pair);

        Extern::Math {
            inner: src.into(),
            is_block: lvl.len() != 1,
        }
    }

    fn parse_code(&mut self, pair: Pair<Rule>) -> Extern {
        let (lvl, src) = self.block_parser(pair);

        Extern::Code {
            lvl: lvl.len(),
            inner: src.into(),
        }
    }

    fn parse_verbatim(&mut self, pair: Pair<Rule>) -> Extern {
        let value = pair.as_str();
        Extern::Verbatim(value.into())
    }

    fn parse_meta(&mut self, pair: Pair<Rule>) {
        self.meta = Some(pair.as_str().to_string());
    }
}

pub fn parse_to_doc(input: &str) -> Result<RawDocument, Error<Rule>> {
    let mut doc = RawDocument::default();
    doc.parse_doc(RawDocParser::parse(Rule::top, input)?)?;
    Ok(doc)
}

fn single_child(pair: Pair<Rule>) -> Pair<Rule> {
    pair.into_inner().next().expect("missing child")
}

#[cfg(test)]
mod tests {
    use crate::common::PosInfo;
    use crate::raw::{
        parse_to_doc, Element, ElementInfo, Extern, Parameter, RawDocument, Reference, Value,
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
                compare(input, doc);
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
                element: Element::Extern(Extern::Code {
                    lvl: 3,
                    inner: "\ncode\n".into(),
                }),
                pos: PosInfo::new(input, 0, 12),
            }],
            meta: None,
            references: Default::default(),
        };

        compare(input, expected);
    }

    #[test]
    fn test_math() {
        let input = "$inline$";
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Extern(Extern::Math {
                    is_block: false,
                    inner: "inline".into(),
                }),
                pos: PosInfo::new(input, 0, 8),
            }],
            meta: None,
            references: Default::default(),
        };

        compare(input, expected);
    }

    #[test]
    fn test_verbatim() {
        let input = "\\{verbatim\\}";
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Extern(Extern::Verbatim("verbatim".into())),
                pos: PosInfo::new(input, 2, 10),
            }],
            meta: None,
            references: Default::default(),
        };

        compare(input, expected);
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

        compare(input, expected);
    }

    #[test]
    fn test_refs() {
        let input = "#call|id";
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Extern(Extern::Command {
                    function: "call".to_string(),
                    id: Some("id".to_string()),
                    parameters: vec![],
                    body: None,
                }),
                pos: PosInfo::new(input, 0, 8),
            }],
            meta: None,
            references: HashMap::from([(
                "id".to_string(),
                Reference::Command("call".to_string(), vec![]),
            )]),
        };

        compare(input, expected);
    }

    fn make_doc(src: Vec<ElementInfo>) -> RawDocument {
        RawDocument {
            src,
            meta: None,
            references: Default::default(),
        }
    }

    const CMD_WITH_PARAMS_NO_BODY: &str =
        "#func(basic, \"quoted\", {content}, key=basic, key=\"quoted\", key={content}, :flag)";

    doc_tests! {
        command
        no_params_no_body: ("#func",  vec![
            ElementInfo {
                element: Element::Extern(Extern::Command {
                    function: "func".into(),
                    id: None,
                    parameters: vec![],
                    body: None,
                }),
                pos: PosInfo::new("#func", 0, 5),
            }
        ]),
        with_params_no_body: (CMD_WITH_PARAMS_NO_BODY,  vec![
            ElementInfo {
                element: Element::Extern(Extern::Command {
                    function: "func".into(),
                    id: None,
                    parameters: vec![
                        Parameter { key: None, value: Value::String("basic".into()), span: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 6, 11) },
                        Parameter { key: None, value: Value::String("quoted".into()), span: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 13, 21) },
                        Parameter { key: None, value: Value::Content(vec![
                            ElementInfo {
                                element: Element::Markdown("content".into()),
                                pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 24, 31)
                            }
                        ]), span: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 23, 32) },
                        Parameter { key: Some("key".into()), value: Value::String("basic".into()), span: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 34, 43) },
                        Parameter { key: Some("key".into()), value: Value::String("quoted".into()), span: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 45, 57) },
                        Parameter { key: Some("key".into()), value: Value::Content(vec![
                            ElementInfo {
                                element: Element::Markdown("content".into()),
                                pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 64, 71)
                            }
                        ]), span: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 59, 72) },
                        Parameter { key: None, value: Value::Flag("flag".into()), span: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 74, 79) }
                    ],
                    body: None,
                }),
                pos: PosInfo::new(CMD_WITH_PARAMS_NO_BODY, 0, 80),
            }
        ]),
        with_params_with_body: ("#func(c){x}", vec![
            ElementInfo {
                element: Element::Extern(Extern::Command {
                    function: "func".into(),
                    id: None,
                    parameters: vec![
                        Parameter { key: None, value: Value::String("c".to_string()), span: PosInfo::new("#func(c){x}", 6, 7)}
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
                element: Element::Extern(Extern::Command {
                    function: "func".into(),
                    id: None,
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
                element: Element::Extern(Extern::Command {
                    function: "func1".into(),
                    id: None,
                    parameters: vec![],
                    body: Some(vec![ElementInfo {
                            element: Element::Extern(Extern::Command{
                                function: "func2".into(),
                                id: None,
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

    fn compare(input: &str, expected: RawDocument) {
        let doc = parse_to_doc(input).expect("Parse error");

        assert_eq!(expected, doc);
    }
}
