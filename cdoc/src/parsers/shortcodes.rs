use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[grammar = "parsers/shortcodes.pest"]
pub struct ShortCodeParser;

/// Represents a shortcode markup element (a call).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortCodeCall {
    pub name: String,
    pub id: Option<String>,
    /// The passed arguments
    pub arguments: Vec<Argument<String>>,
}

/// Value of a shortcode argument.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ArgumentValue<T> {
    // A regular value that is used literally. Always contains a single Block::Plain(Inline::Text(...)) element.
    Literal(T),
    // A value that is parsed as markdown with shortcodes. Useful for rich text such as captions.
    Markdown(T),
    // List(Vec<ArgumentValue<T>>),
}

/// A shortcode argument
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Argument<T> {
    /// No key provided - it is inferred from its position in the vector of arguments and the names
    /// specified in the template definition file.
    Positional { value: ArgumentValue<T> },
    /// Regular keyword argument.
    Keyword {
        name: String,
        value: ArgumentValue<T>,
    },
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
            Argument::Positional { value } => Argument::Positional { value: f(value) },
            Argument::Keyword { name, value } => Argument::Keyword {
                name,
                value: f(value),
            },
        }
    }
    pub fn try_map<U, F: FnMut(ArgumentValue<T>) -> anyhow::Result<ArgumentValue<U>>>(
        self,
        mut f: F,
    ) -> anyhow::Result<Argument<U>> {
        Ok(match self {
            Argument::Positional { value } => Argument::Positional { value: f(value)? },
            Argument::Keyword { name, value } => Argument::Keyword {
                name,
                value: f(value)?,
            },
        })
    }
    pub fn get_value(&self) -> &ArgumentValue<T> {
        match self {
            Argument::Positional { value } => value,
            Argument::Keyword { value, .. } => value,
        }
    }
}

// Fetch parameter value. There are several types to allow for different kinds of escapes.
fn get_value(val: &Pair<Rule>) -> ArgumentValue<String> {
    let v = val.clone().into_inner().next().unwrap();
    match v.as_rule() {
        // Literal string (using quotes `" "` as delimiters)
        Rule::string_val => ArgumentValue::Literal(v.as_str().to_string()),
        // A more restricted literal passed without quotes.
        Rule::basic_val => ArgumentValue::Literal(v.as_str().to_string()),
        // A markdown literal (can include shortcode calls). Delimited by quotes as literals but prepended by a |.
        Rule::md_val => ArgumentValue::Markdown(v.as_str().to_string()),
        _ => unreachable!(),
    }
}

pub(crate) fn parse_shortcode(
    content: &str,
) -> Result<ShortCodeCall, Box<pest::error::Error<Rule>>> {
    let padded = content.to_string();
    let p = ShortCodeParser::parse(Rule::p, &padded)?;

    // Fetch the name of the shortcode
    let mut iter = p;
    let name = iter.next().expect("Missing name").as_str().to_string();
    let mut id = None;

    let mut parameters = Vec::new();

    for params in iter {
        match params.as_rule() {
            Rule::id => id = Some(params.as_str().to_string()), // If the id is present, save it
            Rule::parameters => {
                // Then go through each parameter
                for p in params.into_inner() {
                    match p.as_rule() {
                        Rule::param => {
                            // Fetch parameter inner
                            let mut inner = p.into_inner();
                            let v = inner.next().unwrap();
                            match v.as_rule() {
                                // Parameter format
                                Rule::key => {
                                    // Named argument
                                    // Get key
                                    let k = v.as_str().to_string();
                                    // Get value
                                    let value_pair = inner.next().unwrap();
                                    let value = get_value(&value_pair);
                                    parameters.push(Argument::Keyword { name: k, value })
                                }
                                Rule::value => {
                                    // Positional argument
                                    parameters.push(Argument::Positional {
                                        value: get_value(&v),
                                    });
                                }
                                _ => unreachable!(),
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Rule::EOI => {}
            _ => unreachable!(),
        };
    }

    Ok(ShortCodeCall {
        name,
        id,
        arguments: parameters,
    })
}
