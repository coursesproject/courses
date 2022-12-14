//! This is the module docs

#[macro_use]
extern crate pest_derive;
extern crate core;

pub mod cfg;
mod document;
pub mod document_parser;
pub mod extensions;
pub mod notebook;
pub mod notebook_writer;
pub mod parser;
pub mod parsers;
pub mod pipeline;
mod preprocessor;
pub mod render;
mod visitor;
mod ast;

#[cfg(test)]
mod tests {
    use crate::parsers::split::parse_code_string;
    use crate::parsers::split_types::Output;

    #[test]
    fn test_parse() {
        let str = include_str!("../resources/test/sample.py");
        let _doc = parse_code_string(str).unwrap();
    }

    #[test]
    fn test_output() {
        let str = include_str!("../resources/test/sample.rs");
        let doc = parse_code_string(str).unwrap();

        let _output_solution = doc.write_string(true);
        let _output_placeholder = doc.write_string(false);
    }

    #[test]
    fn test_serialize() {
        let str = include_str!("../resources/test/sample.rs");
        let doc = parse_code_string(str).unwrap();

        let _res = serde_json::to_string(&doc).unwrap();
    }
}
