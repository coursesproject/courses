use crate::loader::Loader;
use crate::parser::Parser;
use crate::parsers::split_types;
use crate::renderers::Renderer;
use std::collections::HashMap;

pub mod ast;
pub mod config;
pub mod document;
pub mod loader;
pub mod notebook;
pub mod parser;
pub mod parsers;
pub mod processors;
pub mod renderers;

type Context = HashMap<String, split_types::Value>;
