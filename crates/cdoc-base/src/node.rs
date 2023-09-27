use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

pub struct Document {
    content: Vec<Node>,
    meta: HashMap<String, Value>,
    data: HashMap<String, DataValue>,
}

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

pub struct Node {
    pub type_id: String,
    pub id: String,
    pub attributes: BTreeMap<String, Attribute>,
    pub children: Option<Vec<Node>>,
    pub value: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub enum Attribute {
    Int(i64),
    Float(f64),
    String(String),
    Enum(String),
    Flag,
}

pub struct NodeTypeDef {
    type_id: String,
    attributes: Option<Vec<AttributeDef>>,
    children: Option<Vec<NodeChildSpec>>,
    has_value: bool,
}

pub struct NodeChildSpec {
    type_: ChildType,
    rule: ChildRule,
}

pub enum ChildType {
    Any,
    Is(String),
    OneOf(Vec<ChildType>),
}

pub enum ChildRule {
    One,
    OneOrMany,
    ZeroOrMany,
    ZeroOrOne,
    Exactly(usize),
}

pub struct AttributeDef {
    name: String,
    optional: bool,
    data_type: DataType,
}

pub enum DataType {
    Int,
    Float,
    String,
    Enum(Vec<String>),
    Flag,
}
