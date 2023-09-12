use crate::raw::RawDocument;
use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammars/raw_doc.pest"]
pub struct RawDocParser;

mod parser {
    use super::*;
    use crate::raw::{Element, ElementInfo, Parameter, Value};
    use pest::iterators::Pairs;
    use std::borrow::Cow;
    pub fn parse_to_doc(input: &str) -> Result<RawDocument, Error<Rule>> {
        parse_doc(RawDocParser::parse(Rule::top, input)?)
    }

    fn parse_doc(mut pairs: Pairs<Rule>) -> Result<RawDocument, Error<Rule>> {
        let mut elems = pairs.next().expect("no root item").into_inner();

        let mut doc = RawDocument {
            src: vec![],
            meta: None,
        };

        if let Some(p) = elems.next() {
            match p.as_rule() {
                Rule::meta => doc.meta = Some(parse_meta(p)?),
                _ => doc.src.push(parse_element(p)),
            }
        }

        doc.src.extend(parse_elements(elems));

        Ok(doc)
    }

    fn single_child(pair: Pair<Rule>) -> Pair<Rule> {
        pair.into_inner().next().expect("missing child")
    }

    fn parse_elements(pairs: Pairs<Rule>) -> impl Iterator<Item = ElementInfo> {
        pairs.map(parse_element)
    }

    fn parse_element(pair: Pair<Rule>) -> ElementInfo {
        let span = pair.as_span();

        let element = match pair.as_rule() {
            Rule::command => parse_command(pair),
            Rule::math_block => parse_math(pair),
            Rule::code_def => parse_code(pair),
            Rule::verbatim => parse_verbatim(pair),
            Rule::src | Rule::string | Rule::body => parse_src(pair),
            _ => unreachable!(),
        };

        ElementInfo { element, span }
    }

    fn parse_src(pair: Pair<Rule>) -> Element {
        let value = pair.as_str();
        Element::Markdown(value.into())
    }

    fn parse_command(pair: Pair<Rule>) -> Element {
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

        if let Some(elem) = inner.next() {
            match elem.as_rule() {
                Rule::parameters => parameters = parse_parameters(elem.into_inner()),
                Rule::body_def => body = Some(parse_elements(elem.into_inner()).collect()),
                Rule::id => id = Some(elem.into_inner().as_str().into()),
                _ => unreachable!(),
            }
        }

        Element::Command {
            function: name.into(),
            id,
            parameters,
            body,
        }
    }

    fn parse_parameters(pairs: Pairs<Rule>) -> Vec<Parameter> {
        pairs
            .into_iter()
            .map(|elem| {
                if let Rule::param = elem.as_rule() {
                    parse_param(elem)
                } else {
                    unreachable!()
                }
            })
            .collect()
    }

    fn parse_param(pair: Pair<Rule>) -> Parameter {
        let span = pair.as_span();
        let mut pairs = pair.into_inner();
        let first = pairs.next().expect("empty param");

        if let Rule::key = first.as_rule() {
            let value = pairs.next().expect("no value");
            Parameter::with_key(first.as_str(), parse_value(value), span)
        } else {
            Parameter::with_value(parse_value(first), span)
        }
    }

    fn parse_value(pair: Pair<Rule>) -> Value {
        match pair.as_rule() {
            Rule::basic_val | Rule::string => Value::String(pair.as_str().into()),
            Rule::md_val => Value::Content(parse_elements(pair.into_inner()).collect()),
            Rule::flag => Value::Flag(pair.as_str().into()),
            _ => unreachable!(),
        }
    }

    fn block_parser(pair: Pair<Rule>) -> (&str, &str) {
        let mut inner = pair.into_inner();
        let lvl = inner.next().expect("missing code_lvl").as_str();
        let src = inner.next().expect("missing code_src").as_str();
        (lvl, src)
    }

    fn parse_math(pair: Pair<Rule>) -> Element {
        let (lvl, src) = block_parser(pair);

        Element::Math {
            inner: src.into(),
            is_block: lvl.len() != 1,
        }
    }

    fn parse_code(pair: Pair<Rule>) -> Element {
        let (lvl, src) = block_parser(pair);

        Element::Code {
            lvl: lvl.len(),
            inner: src.into(),
        }
    }

    fn parse_verbatim(pair: Pair<Rule>) -> Element {
        let value = pair.as_str();
        Element::Verbatim(value.into())
    }

    fn parse_meta(pair: Pair<Rule>) -> Result<Cow<str>, Error<Rule>> {
        Ok(Cow::from(pair.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use crate::raw::doc::parser::parse_to_doc;
    use crate::raw::{Element, ElementInfo, Parameter, RawDocument, Value};
    use pest::Span;

    fn elem(elem: Element) -> ElementInfo {
        ElementInfo {
            element: elem,
            span: Span::new("", 0, 0).unwrap(),
        }
    }

    macro_rules! doc_tests {
        ($prefix:ident $($name:ident: $value:expr,)*) => {
        $(
            paste::item!{
            #[test]
            fn [<$prefix _ $name>]() {
                let (input, expected) = $value;
                let doc = RawDocument { src: expected, meta: None };
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
                element: Element::Code {
                    lvl: 3,
                    inner: "\ncode\n".into(),
                },
                span: Span::new(input, 0, 12).unwrap(),
            }],
            meta: None,
        };

        compare(input, expected);
    }

    #[test]
    fn test_math() {
        let input = "$inline$";
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Math {
                    is_block: false,
                    inner: "inline".into(),
                },
                span: Span::new(input, 0, 8).unwrap(),
            }],
            meta: None,
        };

        compare(input, expected);
    }

    #[test]
    fn test_verbatim() {
        let input = "\\{verbatim\\}";
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Verbatim("verbatim".into()),
                span: Span::new(input, 2, 10).unwrap(),
            }],
            meta: None,
        };

        compare(input, expected);
    }

    #[test]
    fn test_src() {
        let input = "just some stuff {} xx--^*# fsdf";
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Markdown(input.into()),
                span: Span::new(input, 0, 31).unwrap(),
            }],
            meta: None,
        };

        compare(input, expected);
    }

    fn make_doc(src: Vec<ElementInfo>) -> RawDocument {
        RawDocument { src, meta: None }
    }

    const CMD_WITH_PARAMS_NO_BODY: &str =
        "#func(basic, \"quoted\", {content}, key=basic, key=\"quoted\", key={content}, :flag)";

    doc_tests! {
        command
        no_params_no_body: ("#func",  vec![
            ElementInfo {
                element: Element::Command {
                    function: "func".into(),
                    id: None,
                    parameters: vec![],
                    body: None,
                },
                span: Span::new("#func", 0, 5).unwrap(),
            }
        ]),
        with_params_no_body: (CMD_WITH_PARAMS_NO_BODY,  vec![
            ElementInfo {
                element: Element::Command {
                    function: "func".into(),
                    id: None,
                    parameters: vec![
                        Parameter { key: None, value: Value::String("basic".into()), span: Span::new(CMD_WITH_PARAMS_NO_BODY, 6, 11).unwrap() },
                        Parameter { key: None, value: Value::String("quoted".into()), span: Span::new(CMD_WITH_PARAMS_NO_BODY, 13, 21).unwrap() },
                        Parameter { key: None, value: Value::Content(vec![
                            ElementInfo {
                                element: Element::Markdown("content".into()),
                                span:Span::new(CMD_WITH_PARAMS_NO_BODY, 24, 31).unwrap()
                            }
                        ]), span: Span::new(CMD_WITH_PARAMS_NO_BODY, 23, 32).unwrap() },
                        Parameter { key: Some("key".into()), value: Value::String("basic".into()), span: Span::new(CMD_WITH_PARAMS_NO_BODY, 34, 43).unwrap() },
                        Parameter { key: Some("key".into()), value: Value::String("quoted".into()), span: Span::new(CMD_WITH_PARAMS_NO_BODY, 45, 57).unwrap() },
                        Parameter { key: Some("key".into()), value: Value::Content(vec![
                            ElementInfo {
                                element: Element::Markdown("content".into()),
                                span:Span::new(CMD_WITH_PARAMS_NO_BODY, 64, 71).unwrap()
                            }
                        ]), span: Span::new(CMD_WITH_PARAMS_NO_BODY, 59, 72).unwrap() },
                        Parameter { key: None, value: Value::Flag("flag".into()), span: Span::new(CMD_WITH_PARAMS_NO_BODY, 74, 79).unwrap() }
                    ],
                    body: None,
                },
                span: Span::new(CMD_WITH_PARAMS_NO_BODY, 0, 80).unwrap(),
            }
        ]),
        no_params_with_body: ("#func{x}", vec![
            ElementInfo {
                element: Element::Command {
                    function: "func".into(),
                    id: None,
                    parameters: vec![],
                    body: Some(vec![
                        ElementInfo {
                            element: Element::Markdown("x".into()),
                            span: Span::new("#func{x}", 6, 7).unwrap()
                        }
                    ])
                },
                span: Span::new("#func{x}", 0, 8).unwrap(),
            }
        ]),
        body_nested: ("#func1{#func2}", vec![
            ElementInfo {
                element: Element::Command {
                    function: "func1".into(),
                    id: None,
                    parameters: vec![],
                    body: Some(vec![ElementInfo {
                            element: Element::Command{
                                function: "func2".into(),
                                id: None,
                                parameters: vec![],
                                body: None,
                            },
                            span: Span::new("#func1{#func2}", 7, 13).unwrap(),
                        }
                    ])
                },
                span: Span::new("#func1{#func2}", 0, 14).unwrap(),
            }

        ]),
    }

    fn compare(input: &str, expected: RawDocument) {
        let doc = parse_to_doc(input).expect("Parse error");

        assert_eq!(expected, doc);
    }
}
