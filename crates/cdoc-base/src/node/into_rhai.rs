// use crate::node::Element;
// use rhai::{ASTNode, Expr, FnCallExpr, FnCallHashes, Position, Stmt};
//
// impl From<Element> for Stmt {
//     fn from(value: Element) -> Self {
//         match value {
//             Element::Plain(s) => {
//                 let call = FnCallExpr {
//                     namespace: Default::default(),
//                     name: "text".into(),
//                     hashes: FnCallHashes::from_hash(0),
//                     args: Box::new([Expr::StringConstant(s.into(), Position::new(0, 0))]),
//                     capture_parent_scope: false,
//                     op_token: (),
//                 };
//
//                 Stmt::FnCall(call, Position::new(0, 0))
//             }
//             Element::Node(node) => {
//                 let call = FnCallExpr {
//                     namespace: Default::default(),
//                     name: node.type_id.into(),
//                     hashes: FnCallHashes::from_hash(0),
//                     args: Box::new([Expr::StringConstant(s.into(), Position::new(0, 0))]),
//                     capture_parent_scope: false,
//                     op_token: (),
//                 };
//
//                 Stmt::FnCall(call, Position::new(0, 0))
//             }
//         }
//     }
// }

// use crate::node::Attribute;
// use rhai::Dynamic;
//
// impl From<Attribute> for Dynamic {
//     fn from(value: Attribute) -> Self {
//         match value {
//             Attribute::Int(v) | Attribute::Float(v) | Attribute::String(v) | Attribute::Enum(v) => {
//             }
//             Attribute::Compound(_) => {}
//             Attribute::Flag => {}
//         }
//     }
// }

use crate::node::{Element, Node};
use rhai::{Array, Dynamic, Engine};
use std::any::TypeId;

impl Into<Element> for &'_ Dynamic {
    fn into(self) -> Element {
        match self.type_name() {
            "array" => {
                let a: Array = self.clone().into_array().unwrap();
                let elems: Vec<Element> = a.into_iter().map(|e| e.cast::<Element>()).collect();
                Element::Node(Node {
                    type_id: "plain".to_string(),
                    attributes: Default::default(),
                    children: Some(elems),
                })
            }
            _ => Element::Plain(self.to_string()),
        }
    }
}

// pub struct ElementVec(Vec<Element>);
// pub struct ElementChildren(Vec<ElementVec>);

pub fn build_types(engine: &mut Engine) {
    engine.register_type::<Element>();
    // engine.register_type::<ElementChildren>();
}
