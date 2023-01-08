use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;
use crate::processors::shortcodes::ShortCode;

#[derive(Parser)]
#[grammar = "processors/shortcodes/grammar.pest"]
pub struct ShortCodeParser;

pub fn parse_shortcode(content: &str) -> Result<ShortCode, Box<pest::error::Error<Rule>>> {
    let padded = content.to_string();
    let p = ShortCodeParser::parse(Rule::p, &padded)?;

    let mut iter = p;
    let name = iter.next().expect("Missing name").as_str().to_string();

    let mut parameters = HashMap::new();

    match iter.next() {
        None => {}
        Some(params) => {
            for p in params.into_inner() {
                match p.as_rule() {
                    Rule::param => {
                        let mut inner = p.into_inner();
                        let key = inner.next().expect("Missing key").as_str().to_string();

                        let value = inner
                            .next()
                            .expect("Missing value")
                            .into_inner()
                            .next()
                            .expect("Missing value inner");

                        let value = match value.as_rule() {
                            Rule::string_val => value.as_str(),
                            Rule::basic_val => value.as_str(),
                            _ => unreachable!(),
                        };
                        parameters.insert(key, value.to_string());
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    Ok(ShortCode { name, parameters })
}
