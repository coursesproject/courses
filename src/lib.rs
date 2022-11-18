//! This is the module docs


#[macro_use]
extern crate pest_derive;

mod ast;
pub mod cfg;
mod cfg2;
mod document;
pub mod document_parser;
pub mod extensions;
pub mod notebook;
pub mod notebook_writer;
pub mod parser;
pub mod parsers;
pub mod pipeline;
pub mod render;
mod visitor;
mod preprocessor;

#[cfg(test)]
mod tests {
    use crate::parsers::split::parse_code_string;
    use crate::parsers::split_types::Output;

    #[test]
    fn test_parse() {
        let str = include_str!("../resources/test/sample.py");
        let doc = parse_code_string(str).unwrap();
        println!("{:?}", doc);
    }

    #[test]
    fn test_output() {
        let str = include_str!("../resources/test/sample.rs");
        let doc = parse_code_string(str).unwrap();

        let output_solution = doc.write_string(true);
        let output_placeholder = doc.write_string(false);
        println!("{:?}", output_solution);
        println!("{:?}", output_placeholder);
    }

    #[test]
    fn test_serialize() {
        let str = include_str!("../resources/test/sample.rs");
        let doc = parse_code_string(str).unwrap();

        let res = serde_json::to_string(&doc).unwrap();
        println!("{:#?}", res);
    }
}
