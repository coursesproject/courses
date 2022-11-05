use crate::parsers::split::parse_value;
use crate::parsers::split_types::{CodeTaskDefinition, Value};
use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "parsers/shortcodes.pest"]
pub struct ShortCodeParser;

pub struct ShortCode {
    pub(crate) name: String,
    pub(crate) parameters: HashMap<String, String>,
}

pub fn parse_shortcode(content: &str) -> Option<ShortCode> {
    let mut padded = content.to_string();
    padded.push_str("\n");
    let p = ShortCodeParser::parse(Rule::p, &padded).ok()?;

    let mut iter = p.into_iter();
    let name = iter.next()?.as_str().to_string();

    let mut parameters = HashMap::new();

    match iter.next() {
        None => {}
        Some(params) => {
            for p in params.into_inner() {
                match p.as_rule() {
                    Rule::param => {
                        let mut inner = p.into_inner();
                        let key = inner.next()?.as_str().to_string();
                        let value = inner.next()?.as_str().to_string();
                        parameters.insert(key, value);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    Some(ShortCode { name, parameters })
}
