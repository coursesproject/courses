mod utils;

use cdoc_parser::document::Document;
use std::convert::TryFrom;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, cdoc-wasm!");
}

#[wasm_bindgen]
pub fn parse_doc(input: &str) -> JsValue {
    let doc = Document::try_from(input).unwrap();

    serde_wasm_bindgen::to_value(&doc).unwrap()
}
