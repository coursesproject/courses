use crate::raw::{RawDocument, Reference};
use cowstr::CowStr;
use nanoid::nanoid;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Parser)]
#[grammar = "grammars/raw_doc.pest"]
pub struct RawDocParser;

use crate::code_ast::parse_code_string;
use crate::common::Span;
use crate::raw::{ArgumentVal, Element, ElementInfo, Parameter, Special};
use pest::iterators::Pairs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("code cell parsing error")]
    CodeError(#[from] Box<pest::error::Error<crate::code_ast::Rule>>),
    #[error("document parsing error")]
    DocError(#[from] Box<pest::error::Error<Rule>>),
    #[error("integer parsing error")]
    ParseIntError(#[from] ParseIntError),
    #[error("float parsing error")]
    ParseFloatError(#[from] ParseFloatError),
}

const ALPHABET: [char; 16] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
];

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
        let span = Span::from(pair.as_span());

        let element = match pair.as_rule() {
            Rule::command => self.parse_command(pair)?,
            Rule::script => self.parse_script(pair)?,
            Rule::math_block => self.parse_math_block(pair),
            Rule::code_def => self.parse_code(pair)?,
            Rule::verbatim => self.parse_verbatim(pair),
            Rule::src | Rule::string | Rule::body => self.parse_src(pair),
            _ => unreachable!(),
        };

        Ok(ElementInfo { element, span })
    }

    fn parse_src(&mut self, pair: Pair<Rule>) -> Element {
        let value = pair.as_str();
        Element::Markdown(value.into())
    }

    fn parse_script(&mut self, pair: Pair<Rule>) -> Result<Element, ParserError> {
        let mut inner = pair.into_inner();
        let kw = inner.next().unwrap().as_str();

        let mut src = String::from(kw);
        let mut children = vec![];

        let id = nanoid!(10, &ALPHABET);
        for elem in inner.next().unwrap().into_inner() {
            match elem.as_rule() {
                Rule::script_src => src.push_str(elem.as_str()),
                Rule::script_escape => {
                    src.push_str(&format!(" e_{}[{}] ", id, children.len()));
                    children.push(self.parse_elements(elem.into_inner())?)
                }
                _ => unreachable!(),
            }
        }

        src.push(';');

        Ok(Element::Special(
            id.clone().into(),
            Special::Script {
                id,
                src: src.into(),
                children,
            },
        ))
    }

    fn parse_command(&mut self, pair: Pair<Rule>) -> Result<Element, ParserError> {
        let mut inner = pair.into_inner();
        let name = inner
            .next()
            .expect("empty command")
            .into_inner()
            .next()
            .unwrap()
            .as_span();
        let name = self.cowstr_from_span(name);

        let mut parameters = vec![];
        let mut body = None;
        let mut label = None;

        for elem in inner {
            match elem.as_rule() {
                Rule::parameters => parameters = self.parse_parameters(elem.into_inner())?,
                Rule::body_def => body = Some(self.parse_elements(elem.into_inner())?),
                Rule::label => {
                    label = Some(self.cowstr_from_span(elem.into_inner().next().unwrap().as_span()))
                }
                _ => unreachable!(),
            }
        }

        if let Some(label) = label.clone() {
            self.references
                .insert(label, Reference::Command(name.clone(), parameters.clone()));
        }

        Ok(Element::Special(
            label.unwrap_or(nanoid!().into()),
            Special::Command {
                function: name,
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
        let span = Span::from(pair.as_span());
        let mut pairs = pair.into_inner();
        let first = pairs.next().expect("empty param");

        Ok(if let Rule::key = first.as_rule() {
            let value = pairs.next().expect("no value");
            Parameter::with_key(first.as_str(), self.parse_value(value)?, span)
        } else {
            Parameter::with_value(self.parse_value(first)?, span)
        })
    }

    fn parse_value(&mut self, pair: Pair<Rule>) -> Result<ArgumentVal, ParserError> {
        Ok(match pair.as_rule() {
            Rule::basic_val | Rule::string => ArgumentVal::String(pair.as_str().into()),
            Rule::md_val => ArgumentVal::Content(self.parse_elements(pair.into_inner())?),
            Rule::flag => ArgumentVal::Flag(pair.as_str().into()),
            Rule::integer => ArgumentVal::Int(pair.as_str().parse()?),
            Rule::float => ArgumentVal::Float(pair.as_str().parse()?),
            _ => unreachable!(),
        })
    }

    fn parse_math_block(&mut self, pair: Pair<Rule>) -> Element {
        let (lvl, src, label) = self.block_parser(pair);

        let src = self.parse_math(src);

        if let Some(label) = label.clone() {
            self.references.insert(label, Reference::Math(src.clone()));
        }

        Element::Special(
            label.unwrap_or(nanoid!().into()),
            Special::Math {
                inner: src,
                is_block: lvl.len() != 1,
            },
        )
    }

    fn parse_code_attributes(&mut self, pairs: Pairs<Rule>) -> Vec<CowStr> {
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

    fn parse_code_attribute(&mut self, pair: Pair<Rule>) -> CowStr {
        let mut pairs = pair.into_inner();
        let first = pairs.next().expect("empty param");

        if let Rule::key = first.as_rule() {
            let value = pairs.next().expect("no value");
            // CodeAttr {
            //     key: Some(first.as_str().to_string()),
            //     value: value.as_str().to_string(),
            // }
            self.cowstr_from_span(value.as_span())
        } else {
            // CodeAttr {
            //     key: None,
            //     value: first.as_str().to_string(),
            // }
            self.cowstr_from_span(first.as_span())
        }
    }

    fn parse_code(&mut self, pair: Pair<Rule>) -> Result<Element, ParserError> {
        let mut inner = pair.into_inner();
        let lvl = inner.next().expect("missing code_lvl").as_str().to_string();

        let maybe_param = inner.next().expect("missing code_src");
        let (src_pair, params) = if let Rule::code_params = maybe_param.as_rule() {
            let attributes = self.parse_code_attributes(maybe_param.into_inner());
            (inner.next().expect("missing code_src"), Some(attributes))
        } else {
            (maybe_param, None)
        };

        let src_span = src_pair.as_span();
        let src = self.cowstr_from_span(src_span);

        let id = inner.next().map(|val| self.cowstr_from_span(val.as_span()));

        if let Some(label) = id.clone() {
            self.references.insert(label, Reference::Code(src.clone()));
        }

        Ok(Element::Special(
            id.unwrap_or(nanoid!().into()),
            if lvl.len() == 1 {
                Special::CodeInline { inner: src }
            } else {
                let content = parse_code_string(src)?;

                Special::CodeBlock {
                    lvl: lvl.len(),
                    inner: content,
                    attributes: params.unwrap_or_default(),
                }
            },
        ))
    }

    fn parse_verbatim(&mut self, pair: Pair<Rule>) -> Element {
        let value = pair.as_str();
        Element::Special(
            CowStr::from(nanoid!()),
            Special::Verbatim {
                inner: value.into(),
            },
        )
    }

    fn parse_meta(&mut self, pair: Pair<Rule>) {
        self.meta = Some(self.cowstr_from_span(pair.as_span()));
    }

    fn cowstr_from_span(&self, span: pest::Span) -> CowStr {
        CowStr::from(&self.input[span.start()..span.end()])
    }

    fn parse_math(&self, pair: Pair<Rule>) -> CowStr {
        match pair.as_rule() {
            Rule::math_chars => self.cowstr_from_span(pair.as_span()),
            Rule::math_block_curly => cowstr::format!(
                "{{{}}}",
                pair.into_inner()
                    .map(|p| self.parse_math(p))
                    .collect::<CowStr>()
            ),
            // | Rule::math_block_bracket
            // | Rule::math_block_paren
            Rule::math_body => pair
                .into_inner()
                .map(|p| self.parse_math(p))
                .collect::<CowStr>(),
            _ => unreachable!(),
        }
    }

    fn block_parser<'a>(&'a self, pair: Pair<'a, Rule>) -> (CowStr, Pair<Rule>, Option<CowStr>) {
        let mut inner = pair.into_inner();
        let lvl = self.cowstr_from_span(inner.next().expect("missing code_lvl").as_span());
        let src = inner.next().expect("missing code_src");
        let id = inner.next().map(|val| self.cowstr_from_span(val.as_span()));
        (lvl, src, id)
    }
}

pub fn parse_to_doc(input: &str) -> Result<RawDocument, ParserError> {
    let mut doc = RawDocument::new(input);
    doc.parse_doc(RawDocParser::parse(Rule::top, input).map_err(Box::new)?)?;
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use crate::code_ast::types::{CodeContent, CodeElem};
    use crate::common::Span;
    use crate::raw::{
        parse_to_doc, ArgumentVal, Element, ElementInfo, Parameter, RawDocument, Reference, Special,
    };
    use cowstr::CowStr;
    use std::collections::HashMap;

    macro_rules! doc_tests {
        ($prefix:ident $($name:ident: $value:expr,)*) => {
        $(
            paste::item!{
            #[test]
            fn [<$prefix _ $name>]() {
                let (input, expected) = $value;
                let doc = RawDocument { input: CowStr::from(input), src: expected, meta: None, references: Default::default() };
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
                    "".into(),
                    Special::CodeBlock {
                        lvl: 3,
                        inner: CodeContent {
                            blocks: vec![CodeElem::Src("\ncode\n\n".into())],
                            meta: Default::default(),
                            hash: 3750657748055546767,
                        },
                        attributes: vec![],
                    },
                ),
                span: Span::new(0, 12),
            }],
            input: CowStr::from(input),
            meta: None,
            references: Default::default(),
        };

        compare(expected, input);
    }

    #[test]
    fn test_code_param() {
        let input = r#"```lang, val
code
```"#;
        let expected = RawDocument {
            src: vec![ElementInfo {
                element: Element::Special(
                    "".into(),
                    Special::CodeBlock {
                        lvl: 3,
                        inner: CodeContent {
                            blocks: vec![CodeElem::Src("code\n\n".into())],
                            meta: Default::default(),
                            hash: 15492099155864206242,
                        },
                        attributes: vec!["lang".into(), "val".into()],
                    },
                ),
                span: Span::new(0, 21),
            }],
            input: CowStr::from(input),
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
                    "".into(),
                    Special::Math {
                        is_block: false,
                        inner: "inline".into(),
                    },
                ),
                span: Span::new(0, 8),
            }],
            input: CowStr::from(input),
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
                    "".into(),
                    Special::Verbatim {
                        inner: "verbatim".into(),
                    },
                ),
                span: Span::new(2, 10),
            }],
            input: CowStr::from(input),
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
                span: Span::new(0, 31),
            }],
            input: CowStr::from(input),
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
                    "id".into(),
                    Special::Command {
                        function: "call".into(),
                        parameters: vec![],
                        body: None,
                    },
                ),
                span: Span::new(0, 8),
            }],
            input: CowStr::from(input),
            meta: None,
            references: HashMap::from([("id".into(), Reference::Command("call".into(), vec![]))]),
        };

        compare(expected, input);
    }

    const CMD_WITH_PARAMS_NO_BODY: &str =
        "#func(basic, \"quoted\", {content}, key=basic, key=\"quoted\", key={content}, :flag)";

    doc_tests! {
        command
        no_params_no_body: ("#func",  vec![
            ElementInfo {
                element: Element::Special("".into(), Special::Command {
                    function: "func".into(),
                    parameters: vec![],
                    body: None,
                }),
                span: Span::new(0, 5),
            }
        ]),
        with_params_no_body: (CMD_WITH_PARAMS_NO_BODY,  vec![
            ElementInfo {
                element: Element::Special("".into(), Special::Command {
                    function: "func".into(),
                    parameters: vec![
                        Parameter { key: None, value: ArgumentVal::String("basic".into()), span: Span::new(6, 11) },
                        Parameter { key: None, value: ArgumentVal::String("quoted".into()), span: Span::new(13, 21) },
                        Parameter { key: None, value: ArgumentVal::Content(vec![
                            ElementInfo {
                                element: Element::Markdown("content".into()),
                                span: Span::new(24, 31)
                            }
                        ]), span: Span::new(23, 32) },
                        Parameter { key: Some("key".into()), value: ArgumentVal::String("basic".into()), span: Span::new(34, 43) },
                        Parameter { key: Some("key".into()), value: ArgumentVal::String("quoted".into()), span: Span::new(45, 57) },
                        Parameter { key: Some("key".into()), value: ArgumentVal::Content(vec![
                            ElementInfo {
                                element: Element::Markdown("content".into()),
                                span: Span::new(64, 71)
                            }
                        ]), span: Span::new( 59, 72) },
                        Parameter { key: None, value: ArgumentVal::Flag("flag".into()), span: Span::new(74, 79) }
                    ],
                    body: None,
                }),
                span: Span::new(0, 80),
            }
        ]),
        with_params_with_body: ("#func(c){x}", vec![
            ElementInfo {
                element: Element::Special("".into(), Special::Command {
                    function: "func".into(),
                    parameters: vec![
                        Parameter { key: None, value: ArgumentVal::String("c".into()), span: Span::new(6, 7)}
                    ],
                    body: Some(vec![
                        ElementInfo {
                            element: Element::Markdown("x".into()),
                            span: Span::new(9, 10)
                        }
                    ])
                }),
                span: Span::new(0, 11),
            }
        ]),
        no_params_with_body: ("#func{x}", vec![
            ElementInfo {
                element: Element::Special("".into(), Special::Command {
                    function: "func".into(),
                    parameters: vec![],
                    body: Some(vec![
                        ElementInfo {
                            element: Element::Markdown("x".into()),
                            span: Span::new(6, 7)
                        }
                    ])
                }),
                span: Span::new(0, 8),
            }
        ]),
        body_nested: ("#func1{#func2}", vec![
            ElementInfo {
                element: Element::Special("".into(), Special::Command {
                    function: "func1".into(),
                    parameters: vec![],
                    body: Some(vec![ElementInfo {
                            element: Element::Special("".into(), Special::Command{
                                function: "func2".into(),
                                parameters: vec![],
                                body: None,
                            }),
                            span: Span::new(7, 13),
                        }
                    ])
                }),
                span: Span::new(0, 14),
            }

        ]),
    }

    fn compare(expected: RawDocument, input: &str) {
        let doc = parse_to_doc(input).expect("Parse error");

        assert_eq!(expected, doc);
    }
}
