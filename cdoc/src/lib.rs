use crate::parsers::split_types;
use std::collections::HashMap;

mod ast;
pub mod document;
mod loader;
pub mod notebook;
pub mod parser;
pub mod parsers;
pub mod processors;
mod renderers;

type Context = HashMap<String, split_types::Value>;
