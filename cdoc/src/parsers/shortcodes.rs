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
    pub parameters: HashMap<String, String>,
}

pub fn parse_shortcode(content: &str) -> Result<ShortCodeDef, Box<pest::error::Error<Rule>>> {
    let padded = content.to_string();
    let p = ShortCodeParser::parse(Rule::p, &padded)?;

    let mut iter = p;
    let name = iter.next().expect("Missing name").as_str().to_string();
    let mut id = None;

    let mut parameters = HashMap::new();

    while let Some(params) = iter.next() {
        match params.as_rule() {
            Rule::id => id = Some(params.as_str().to_string()),
            Rule::parameters => {
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
