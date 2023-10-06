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
pub enum Node {
    Plain(String),
    Compound(Compound),
    Script(Script),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub src: String,
    pub elements: Vec<Vec<Node>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compound {
    pub type_id: String,
    pub attributes: BTreeMap<String, Attribute>,
    pub children: Vec<Node>,
}

impl Compound {
    pub fn new<S: Into<String>, B: IntoIterator<Item = (String, Attribute)>>(
        type_id: S,
        attributes: B,
        children: Vec<Node>,
    ) -> Self {
        Self {
            type_id: type_id.into(),
            attributes: attributes.into_iter().collect(),
            children,
        }
    }

    pub fn new_with_children<S: Into<String>>(type_id: S, children: Vec<Node>) -> Self {
        Self::new(type_id, BTreeMap::new(), children)
    }

    pub fn new_with_attributes<S: Into<String>, B: IntoIterator<Item = (String, Attribute)>>(
        type_id: S,
        attributes: B,
    ) -> Self {
        Self {
            type_id: type_id.into(),
            attributes: attributes.into_iter().collect(),
            children: vec![],
        }
    }

    pub fn new_empty<S: Into<String>>(type_id: S) -> Self {
        Self {
            type_id: type_id.into(),
            attributes: BTreeMap::new(),
            children: vec![],
        }
    }
}

impl Node {
    pub fn get_compound(&self) -> &Compound {
        if let Node::Compound(c) = self {
            c
        } else {
            panic!()
        }
    }

    pub fn get_plain(&self) -> &String {
        if let Node::Plain(s) = self {
            s
        } else {
            panic!()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Attribute {
    Int(i64),
    Float(f64),
    String(String),
    Enum(String),
    Compound(Vec<Node>),
    Flag,
}

impl Attribute {
    pub fn get_string(&self) -> &str {
        if let Attribute::String(s) = self {
            s
        } else {
            panic!()
        }
    }
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
