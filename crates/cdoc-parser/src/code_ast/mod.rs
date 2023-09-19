pub mod types;

use crate::code_ast::types::{CodeContent, CodeElem, Solution};

use linked_hash_map::LinkedHashMap;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// The parser for exercise placeholders/solutions.
#[derive(Parser)]
#[grammar = "grammars/exercises.pest"]
pub struct TaskParser;

pub(crate) fn parse_code_placeholder_block(
    pair: Pair<Rule>,
) -> Result<String, Box<pest::error::Error<Rule>>> {
    match pair.as_rule() {
        Rule::source_comment_block => Ok(pair
            .into_inner()
            .map(parse_source_comment)
            .collect::<anyhow::Result<String, Box<pest::error::Error<Rule>>>>(
        )?),
        _ => unreachable!(),
    }
}

fn parse_source_comment(pair: Pair<Rule>) -> Result<String, Box<pest::error::Error<Rule>>> {
    Ok(match pair.as_rule() {
        Rule::source_comment => {
            let mut inner = pair.into_inner();
            let spacing = inner.next().expect("Unexpected end of iterator");
            let spacing: String = spacing.as_str().parse().expect("String parse error");
            let str = inner.next().expect("Unexpected end of iterator");
            let str: String = str.as_str().parse().expect("String parse error");

            format!("{spacing}{str}")
        }
        _ => unreachable!(),
    })
}

pub(crate) fn parse_value(
    pair: Pair<Rule>,
    meta: &mut LinkedHashMap<String, String>,
) -> Result<Option<CodeElem>, Box<pest::error::Error<Rule>>> {
    Ok(match pair.as_rule() {
        Rule::source_code_block => Some(CodeElem::Src(pair.as_str().to_string())),

        Rule::code_block => {
            let mut block_segments = pair.into_inner();
            let solution_pair = block_segments.next().expect("Unexpected end of iterator");

            let solution = solution_pair.into_inner().as_str().to_string();

            let placeholder: Option<String> = if let Some(placeholder_pair) = block_segments.next()
            {
                Some(
                    placeholder_pair
                        .into_inner()
                        .map(parse_code_placeholder_block)
                        .collect::<anyhow::Result<String, Box<pest::error::Error<Rule>>>>()?,
                )
            } else {
                None
            };

            Some(CodeElem::Solution(Solution {
                placeholder,
                solution,
            }))
        }

        Rule::meta => {
            let mut outer = pair.into_inner();
            let tp = outer.next().expect("Missing meta type");
            let mut content = tp.into_inner();

            let ident = content.next().unwrap().as_str().to_string();
            let value = content.next().unwrap().as_str().to_string();

            meta.insert(ident, value);
            None
        }

        _ => unreachable!(),
    })
}

pub fn parse_code_string(content: &str) -> Result<CodeContent, Box<pest::error::Error<Rule>>> {
    let mut padded = content.to_string();
    padded.push('\n');
    let mut p = TaskParser::parse(Rule::doc, &padded)?;
    let p = p.next().expect("no top level").into_inner();
    let mut meta = LinkedHashMap::new();
    let blocks = p
        .into_iter()
        .filter_map(|v| parse_value(v, &mut meta).transpose())
        .collect::<anyhow::Result<Vec<CodeElem>, Box<pest::error::Error<Rule>>>>()?;
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    Ok(CodeContent {
        blocks,
        meta,
        hash: hasher.finish(),
    })
}

pub(crate) fn human_errors(error: pest::error::Error<Rule>) -> Box<pest::error::Error<Rule>> {
    Box::new(error.renamed_rules(|rule| match *rule {
        Rule::source_code_block => "code".to_owned(),
        Rule::code_block => "placeholder/solution block".to_owned(),

        Rule::source_comment => "code comment".to_owned(),
        Rule::source_comment_block => "code comment".to_owned(),

        _ => "Unknown".to_owned(),
    }))
}

pub fn format_pest_err(error: pest::error::Error<Rule>) -> String {
    let error = human_errors(error);
    // format!(
    //     r#"
    // line: {:?}, col: {:?},
    // details: {}
    // "#,
    //     error.location, error.line_col, error.variant
    // )
    format!("{}", error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let str = include_str!("../../resources/sample.py");
        let _doc = parse_code_string(str).unwrap();
    }

    // #[test]
    // fn test_output() {
    //     let str = include_str!("../../../resources/test/sample.rs");
    //     let doc = parse_code_string(str).unwrap();
    //
    //     let _output_solution = doc.write_string(true);
    //     let _output_placeholder = doc.write_string(false);
    // }

    #[test]
    fn test_serialize() {
        let str = include_str!("../../resources/sample.rs");
        let doc = parse_code_string(str).unwrap();

        let _res = serde_json::to_string(&doc).unwrap();
    }
}
