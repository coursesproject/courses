pub mod into_rhai;
pub mod visitor;
pub mod xml_writer;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

// pub struct Document {
//     content: Vec<Element>,
//     meta: HashMap<String, Value>,
//     data: HashMap<String, DataValue>,
// }

pub enum DataValue {
    String { kind: String, value: String },
    Image(Image),
    Json(Value),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Image {
    Png(String),
    Svg(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Element {
    Plain(String),
    Node(Node),
    Script(Script),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub src: String,
    pub elements: Vec<Vec<Element>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub type_id: String,
    pub attributes: BTreeMap<String, Attribute>,
    pub children: Option<Vec<Element>>,
}

impl Node {
    pub fn new<S: Into<String>, B: IntoIterator<Item = (String, Attribute)>>(
        type_id: S,
        attributes: B,
        children: Vec<Element>,
    ) -> Self {
        Self {
            type_id: type_id.into(),
            attributes: attributes.into_iter().collect(),
            children: Some(children),
        }
    }

    pub fn new_with_children<S: Into<String>>(type_id: S, children: Vec<Element>) -> Self {
        Self::new(type_id, BTreeMap::new(), children)
    }

    pub fn new_with_attributes<S: Into<String>, B: IntoIterator<Item = (String, Attribute)>>(
        type_id: S,
        attributes: B,
    ) -> Self {
        Self {
            type_id: type_id.into(),
            attributes: attributes.into_iter().collect(),
            children: None,
        }
    }

    pub fn new_empty<S: Into<String>>(type_id: S) -> Self {
        Self {
            type_id: type_id.into(),
            attributes: BTreeMap::new(),
            children: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Attribute {
    Int(i64),
    Float(f64),
    String(String),
    Enum(String),
    Compound(Vec<Element>),
    Flag,
}

#[derive(Deserialize)]
pub struct NodeTypeDef {
    pub type_id: String,
    pub attributes: Option<Vec<AttributeDef>>,
    pub children: Option<Vec<NodeChildSpec>>,
}

#[derive(Deserialize)]
pub struct NodeChildSpec {
    pub type_: ChildType,
    pub rule: ChildRule,
}

#[derive(Deserialize)]
pub enum ChildType {
    Any,
    Is(String),
    OneOf(Vec<ChildType>),
}

#[derive(Deserialize)]
pub enum ChildRule {
    One,
    OneOrMany,
    ZeroOrMany,
    ZeroOrOne,
    Exactly(usize),
}

#[derive(Deserialize)]
pub struct AttributeDef {
    pub name: String,
    pub optional: bool,
    pub data_type: DataType,
}

#[derive(Deserialize)]
pub enum DataType {
    Int,
    Float,
    String,
    Enum(Vec<String>),
    Flag,
}
