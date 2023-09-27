use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
    pub attributes: Option<Vec<Attribute>>,
    pub children: Option<Vec<Node>>,
}

#[derive(Serialize, Deserialize)]
pub enum Attribute {
    Int(i64),
    Float(f64),
    String(String),
    Node(Vec<Node>),
    Enum(String),
}

pub struct NodeTypeDef {
    type_id: String,
    has_children: bool,
    attributes: Vec<AttributeDef>,
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
    Node,
    Enum(Vec<String>),
}
