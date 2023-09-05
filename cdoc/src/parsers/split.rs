use pest::error::ErrorVariant;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::parsers::split_types::{CodeTaskDefinition, Content, ExerciseBlock, SplitParseValue};

/// The parser for exercise placeholders/solutions.
#[derive(Parser)]
#[grammar = "parsers/exercises.pest"]
pub struct TaskParser;

pub(crate) fn parse_src_block(pair: Pair<Rule>) -> Result<Content, Box<pest::error::Error<Rule>>> {
    Ok(match pair.as_rule() {
        Rule::source_code_block => Content::Code {
            code: pair.as_str().parse().expect("String parse error"),
        },
        _ => unreachable!(),
    })
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

pub(crate) fn parse_code_placeholder_block(
    pair: Pair<Rule>,
) -> Result<Content, Box<pest::error::Error<Rule>>> {
    match pair.as_rule() {
        Rule::source_comment_block => Ok(Content::Code {
            code: pair
                .into_inner()
                .map(parse_source_comment)
                .collect::<anyhow::Result<Vec<String>, Box<pest::error::Error<Rule>>>>()?
                .join("\n"),
        }),
        _ => Err(Box::new(pest::error::Error::new_from_span(
            ErrorVariant::ParsingError {
                positives: vec![Rule::source_comment_block],
                negatives: vec![pair.as_rule()],
            },
            pair.as_span(),
        ))),
    }
}

pub(crate) fn parse_value(
    pair: Pair<Rule>,
) -> Result<SplitParseValue, Box<pest::error::Error<Rule>>> {
    Ok(match pair.as_rule() {
        Rule::source_code_block => SplitParseValue::SrcBlock {
            content: parse_src_block(pair)?,
        },
        Rule::code_block => {
            let mut block_segments = pair.into_inner();
            let solution_pair = block_segments.next().expect("Unexpected end of iterator");

            let solution: Vec<Content> =
                solution_pair
                    .into_inner()
                    .map(parse_src_block)
                    .collect::<anyhow::Result<Vec<Content>, Box<pest::error::Error<Rule>>>>()?;

            let placeholder: Option<Vec<Content>> = if let Some(placeholder_pair) =
                block_segments.next()
            {
                Some(
                    placeholder_pair
                        .into_inner()
                        .map(parse_code_placeholder_block)
                        .collect::<anyhow::Result<Vec<Content>, Box<pest::error::Error<Rule>>>>()?,
                )
            } else {
                None
            };

            SplitParseValue::SolutionBlock(ExerciseBlock {
                placeholder,
                solution,
            })
        }

        Rule::meta => {
            let mut outer = pair.into_inner();
            let tp = outer.next().expect("Missing meta type");
            let mut content = tp.into_inner();

            let ident = content.next().unwrap().as_str().to_string();
            let value = content.next().unwrap().as_str().to_string();

            SplitParseValue::Meta(ident, value)
        }

        _ => unreachable!(),
    })
}

pub fn parse_code_string(
    content: &str,
) -> Result<CodeTaskDefinition, Box<pest::error::Error<Rule>>> {
    let mut padded = content.to_string();
    padded.push('\n');
    let p = TaskParser::parse(Rule::doc, &padded)?;

    let vals = p
        .into_iter()
        .map(parse_value)
        .collect::<anyhow::Result<Vec<SplitParseValue>, Box<pest::error::Error<Rule>>>>()?;
    Ok(CodeTaskDefinition { blocks: vals })
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
        let str = include_str!("../../resources/test/sample.py");
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
        let str = include_str!("../../resources/test/sample.rs");
        let doc = parse_code_string(str).unwrap();

        let _res = serde_json::to_string(&doc).unwrap();
    }
}
