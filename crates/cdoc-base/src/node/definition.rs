use crate::template::TemplateSource;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeDef {
    pub name: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
    pub children: bool,
    pub templates: HashMap<String, String>,
    pub examples: Vec<UsageExample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsageExample {
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Deserialize)]
pub struct ParameterType {}
