use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub enum TemplateSource {
    /// Raw source specified in yml file.
    String(String),
    /// Path to a file that contains the source (useful for large templates).
    File(PathBuf),
    /// Really just uses the exact template of another format
    Derive(String),
}
