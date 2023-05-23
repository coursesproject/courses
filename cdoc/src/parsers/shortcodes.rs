use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[grammar = "parsers/shortcodes.pest"]
pub struct ShortCodeParser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortCodeDef {
    pub name: String,
    pub id: Option<String>,
    // pub parameters: HashMap<String, String>,
    pub parameters: Vec<Argument<String>>,
}

// Value of a shortcode argument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgumentValue<T> {
    // A regular value that is used literally. Always contains a single Block::Plain(Inline::Text(...)) element.
    Literal(T),
    // A value that is parsed as markdown with shortcodes. Useful for rich text such as captions.
    Markdown(T),
}

/// A shortcode argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Argument<T> {
    /// No key provided - it is inferred from its position in the vector of arguments and the names
    /// specified in the template definition file.
    Positional(ArgumentValue<T>),
    /// Regular keyword argument.
    Keyword(String, ArgumentValue<T>),
}

impl<T> ArgumentValue<T> {
    pub fn inner(&self) -> &T {
        match self {
            ArgumentValue::Literal(i) => i,
            ArgumentValue::Markdown(i) => i,
        }
    }

    pub fn map<U, F: FnMut(T) -> U>(self, mut f: F) -> ArgumentValue<U> {
        match self {
            ArgumentValue::Literal(s) => ArgumentValue::Literal(f(s)),
            ArgumentValue::Markdown(s) => ArgumentValue::Literal(f(s)),
        }
    }

    pub fn try_map<U, F: FnMut(T) -> anyhow::Result<U>>(
        self,
        mut f: F,
    ) -> anyhow::Result<ArgumentValue<U>> {
        Ok(match self {
            ArgumentValue::Literal(s) => ArgumentValue::Literal(f(s)?),
            ArgumentValue::Markdown(s) => ArgumentValue::Literal(f(s)?),
        })
    }
}

impl<T> Argument<T> {
    pub fn map<U, F: FnMut(ArgumentValue<T>) -> ArgumentValue<U>>(self, mut f: F) -> Argument<U> {
        match self {
            Argument::Positional(v) => Argument::Positional(f(v)),
            Argument::Keyword(k, v) => Argument::Keyword(k, f(v)),
        }
    }
    pub fn try_map<U, F: FnMut(ArgumentValue<T>) -> anyhow::Result<ArgumentValue<U>>>(
        self,
        mut f: F,
    ) -> anyhow::Result<Argument<U>> {
        Ok(match self {
            Argument::Positional(v) => Argument::Positional(f(v)?),
            Argument::Keyword(k, v) => Argument::Keyword(k, f(v)?),
        })
    }
    pub fn get_value(&self) -> &ArgumentValue<T> {
        match self {
            Argument::Positional(v) => v,
            Argument::Keyword(_, v) => v,
        }
    }
}

fn get_value(val: &Pair<Rule>) -> ArgumentValue<String> {
    let v = val.clone().into_inner().next().unwrap();
    match v.as_rule() {
        Rule::string_val => ArgumentValue::Literal(v.as_str().to_string()),
        Rule::basic_val => ArgumentValue::Literal(v.as_str().to_string()),
        Rule::markdown_string => ArgumentValue::Markdown(v.as_str().to_string()),
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
                                    parameters.push(Argument::Keyword(k, value))
                                }
                                Rule::value => {
                                    parameters.push(Argument::Positional(get_value(&v)));
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
