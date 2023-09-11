pub mod ast;
pub mod doc;

#[cfg(test)]
use pest_test_gen::pest_tests;

#[pest_tests(
    crate::doc::RawDocParser,
    crate::doc::Rule,
    "doc",
    no_eoi = true,
    dir = "tests/pest/doc",
    strict = false,
    lazy_static = true
)]
#[cfg(test)]
mod raw_doc_tests {}
