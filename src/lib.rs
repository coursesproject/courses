//! This is the module docs

extern crate core;

mod generators;
pub mod pipeline;
pub mod project;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
