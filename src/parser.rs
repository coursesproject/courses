use pest::iterators::Pair;
use pest::Parser;
use std::collections::HashMap;

use crate::types::{Block, SolutionBlock, Content, Document, Inner, Value};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct TaskParser;

pub fn parse_markup_block(pair: Pair<Rule>) -> anyhow::Result<String> {
    Ok(pair
        .into_inner()
        .map(|p| Ok(p.as_str().parse()?))
        .collect::<anyhow::Result<Vec<String>>>()?
        .join("\n"))
}

pub fn parse_src_block(pair: Pair<Rule>) -> anyhow::Result<Content> {
    Ok(match pair.as_rule() {
        Rule::source_code_block => Content::Code {
            code: pair.as_str().parse()?,
        },
        Rule::markup_block => Content::Markup {
            markup: parse_markup_block(pair)?,
        },
        _ => unreachable!(),
    })
}

fn parse_source_comment(pair: Pair<Rule>) -> anyhow::Result<String> {
    Ok(match pair.as_rule() {
        Rule::source_comment => {
            let mut inner = pair.into_inner();
            let spacing = inner.next().unwrap();
            let str = inner.next().unwrap();
            let spacing: String = spacing.as_str().parse()?;
            let str: String = str.as_str().parse()?;

            format!("{}{}", spacing, str)
        }
        _ => unreachable!(),
    })
}

pub fn parse_code_placeholder_block(pair: Pair<Rule>) -> anyhow::Result<Content> {
    Ok(match pair.as_rule() {
        Rule::source_comment_block => Content::Code {
            code: pair
                .into_inner()
                .map(parse_source_comment)
                .collect::<anyhow::Result<Vec<String>>>()?
                .join("\n"),
        },
        Rule::markup_block => Content::Markup {
            markup: parse_markup_block(pair)?,
        },
        _ => unreachable!(),
    })
}

pub fn parse_inner_block(pair: Pair<Rule>) -> anyhow::Result<Inner> {
    let r = pair.as_rule();
    Ok(match r {
        Rule::code_block => {
            let mut block_segments = pair.into_inner();
            let placeholder_pair = block_segments.next().unwrap();
            let solution_pair = block_segments.next().unwrap();

            let placeholder: Vec<Content> = placeholder_pair
                .into_inner()
                .map(parse_code_placeholder_block)
                .collect::<anyhow::Result<Vec<Content>>>()?;
            let solution: Vec<Content> = solution_pair
                .into_inner()
                .map(parse_src_block)
                .collect::<anyhow::Result<Vec<Content>>>()?;

            Inner::SolutionBlock(SolutionBlock {
                placeholder,
                solution,
            })
        }
        Rule::source_code_block | Rule::markup_block => Inner::SrcBlock(parse_src_block(pair)?),
        _ => unreachable!(),
    })
}

pub fn parse_attribute(pair: Pair<Rule>) -> anyhow::Result<(String, String)> {
    Ok(match pair.as_rule() {
        Rule::attr => {
            let mut segments = pair.into_inner();
            let name = segments.next().unwrap().as_str().parse()?;
            let value = segments.next().unwrap().as_str().parse()?;

            (name, value)
        }
        _ => unreachable!(),
    })
}

pub fn parse_value(pair: Pair<Rule>) -> anyhow::Result<Value> {
    Ok(match pair.as_rule() {
        Rule::block => {
            let mut block_segments = pair.into_inner();
            let keyword = block_segments.next().unwrap().as_str().parse()?;
            let attribute_pairs = block_segments.next().unwrap();
            let content_pairs = block_segments.next().unwrap();

            let attributes_vec = attribute_pairs
                .into_inner()
                .map(parse_attribute)
                .collect::<anyhow::Result<Vec<(String, String)>>>()?;
            let content = content_pairs
                .into_inner()
                .map(parse_inner_block)
                .collect::<anyhow::Result<Vec<Inner>>>()?;

            Value::Block {
                block: Block {
                    keyword,
                    attributes: HashMap::from_iter(attributes_vec),
                    inner: content,
                },
            }
        }
        Rule::source_code_block | Rule::markup_block => Value::SrcBlock {
            content: parse_src_block(pair)?,
        },
        Rule::comment_def => Value::SrcBlock {
            content: Content::Markup {
                markup: pair.as_str().parse()?,
            },
        },
        _ => unreachable!(),
    })
}

pub fn parse_document(content: &str) -> anyhow::Result<Document> {
    let p = TaskParser::parse(Rule::doc, content)?;

    let vals = p
        .into_iter()
        .map(parse_value)
        .collect::<anyhow::Result<Vec<Value>>>()?;
    Ok(Document { blocks: vals })
}
