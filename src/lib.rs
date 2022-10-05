#[macro_use]
extern crate pest_derive;

pub mod parser;
mod types;
pub mod notebook;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{parse_document, parse_inner_block, parse_src_block};
    use crate::types::Output;

    #[test]
    fn test_parse() {
        let str = include_str!("../resources/test/sample.py");
        let doc = parse_document(str).unwrap();
        for _ in 0..100 {
            let doc = parse_document(str).unwrap();
        }
        println!("{:?}", doc);
    }

    #[test]
    fn test_output() {
        let str = include_str!("../resources/test/sample.rs");
        let doc = parse_document(str).unwrap();

        let output_solution = doc.to_string(true);
        let output_placeholder = doc.to_string(false);
        println!("{:?}", output_solution);
    }

    #[test]
    fn test_serialize() {
        let str = include_str!("../resources/test/sample.rs");
        let doc = parse_document(str).unwrap();

        let res = serde_json::to_string(&doc).unwrap();
        println!("{:#?}", res);
    }
}
