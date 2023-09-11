use crate::ast::RawDocument;
use pest::error::Error;
use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammars/raw_doc.pest"]
pub struct RawDocParser;

impl RawDocParser {
    // fn parse_doc(pair: Pair<Rule>) -> Result<RawDocument, Error<Rule>> {
    //     let mut elems = pair.into_inner();
    //
    //     let mut doc = RawDocument {
    //         src: vec![],
    //         meta: None,
    //     }
    //
    //     if let Some(p) = elems.next() {
    //         match p {
    //             Rule::meta => doc.meta(p.into_inner()),
    //             Rule::element => doc.src.push()
    //         }
    //     }
    //
    //     for elem in elems {
    //         match elem {
    //             Rule::meta => {}
    //             Rule::element => {}
    //             _ => unreachable!(),
    //         }
    //     }
    // }
    //
    // fn parse_meta(pair: Pair<Rule>) -> Result<String, Error<Rule>>
}
