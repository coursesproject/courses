use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "parsers/shortcodes.pest"]
pub struct ShortCodeParser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortCodeDef {
    pub name: String,
    pub id: Option<String>,
    // pub parameters: HashMap<String, String>,
    pub parameters: Vec<Parameter<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamValue<T> {
    Literal(T),
    Markdown(T),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Parameter<T> {
    Positional(ParamValue<T>),
    Keyword(String, ParamValue<T>),
}

impl<T> ParamValue<T> {
    pub fn inner(&self) -> &T {
        match self {
            ParamValue::Literal(i) => i,
            ParamValue::Markdown(i) => i,
        }
    }

    pub fn map<U, F: FnMut(T) -> U>(self, mut f: F) -> ParamValue<U> {
        match self {
            ParamValue::Literal(s) => ParamValue::Literal(f(s)),
            ParamValue::Markdown(s) => ParamValue::Literal(f(s)),
        }
    }

    pub fn try_map<U, F: FnMut(T) -> anyhow::Result<U>>(
        self,
        mut f: F,
    ) -> anyhow::Result<ParamValue<U>> {
        Ok(match self {
            ParamValue::Literal(s) => ParamValue::Literal(f(s)?),
            ParamValue::Markdown(s) => ParamValue::Literal(f(s)?),
        })
    }
}

impl<T> Parameter<T> {
    pub fn map<U, F: FnMut(ParamValue<T>) -> ParamValue<U>>(self, mut f: F) -> Parameter<U> {
        match self {
            Parameter::Positional(v) => Parameter::Positional(f(v)),
            Parameter::Keyword(k, v) => Parameter::Keyword(k, f(v)),
        }
    }
    pub fn try_map<U, F: FnMut(ParamValue<T>) -> anyhow::Result<ParamValue<U>>>(
        self,
        mut f: F,
    ) -> anyhow::Result<Parameter<U>> {
        Ok(match self {
            Parameter::Positional(v) => Parameter::Positional(f(v)?),
            Parameter::Keyword(k, v) => Parameter::Keyword(k, f(v)?),
        })
    }
    pub fn get_value(&self) -> &ParamValue<T> {
        match self {
            Parameter::Positional(v) => v,
            Parameter::Keyword(_, v) => v,
        }
    }
}

fn get_value(val: &Pair<Rule>) -> ParamValue<String> {
    let v = val.clone().into_inner().next().unwrap();
    match v.as_rule() {
        Rule::string_val => ParamValue::Literal(v.as_str().to_string()),
        Rule::basic_val => ParamValue::Literal(v.as_str().to_string()),
        Rule::markdown_string => ParamValue::Markdown(v.as_str().to_string()),
        _ => unreachable!(),
    }
}

pub fn parse_shortcode(content: &str) -> Result<ShortCodeDef, Box<pest::error::Error<Rule>>> {
    let padded = content.to_string();
    let p = ShortCodeParser::parse(Rule::p, &padded)?;

    let mut iter = p;
    let name = iter.next().expect("Missing name").as_str().to_string();
    let mut id = None;

    let mut parameters = Vec::new();

    for params in iter {
        match params.as_rule() {
            Rule::id => id = Some(params.as_str().to_string()),
            Rule::parameters => {
                for p in params.into_inner() {
                    match p.as_rule() {
                        Rule::param => {
                            let mut inner = p.into_inner();
                            let v = inner.next().unwrap();
                            match v.as_rule() {
                                Rule::key => {
                                    let k = v.as_str().to_string();
                                    let value_pair = inner.next().unwrap();
                                    let value = get_value(&value_pair);
                                    parameters.push(Parameter::Keyword(k, value))
                                }
                                Rule::value => {
                                    parameters.push(Parameter::Positional(get_value(&v)));
                                }
                                _ => unreachable!(),
                            }
                            // let key = inner.next().expect("Missing key").as_str().to_string();
                            //
                            // let value = inner
                            //     .next()
                            //     .and_then(|v| {
                            //         v.into_inner().next().map(|v| match v.as_rule() {
                            //             Rule::string_val => v.as_str(),
                            //             Rule::basic_val => v.as_str(),
                            //             Rule::markdown_val => v.as_str(),
                            //             _ => unreachable!(),
                            //         })
                            //     })
                            //     .unwrap_or("-");
                            //
                            // parameters.insert(key, value.to_string());
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Rule::EOI => {}
            _ => unreachable!(),
        };
    }

    Ok(ShortCodeDef {
        name,
        id,
        parameters,
    })
}
