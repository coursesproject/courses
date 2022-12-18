//! This is the module docs

#[macro_use]
extern crate pest_derive;
extern crate core;

use cdoc::loader::Loader;
use cdoc::parser::Parser;
use cdoc::renderers::Renderer;
use std::collections::HashMap;

mod generators;
pub mod pipeline;
pub mod project;
