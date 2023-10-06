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

use crate::node::{Compound, Node};
use rhai::{Array, Dynamic, Engine};
use std::any::TypeId;

impl Into<Node> for &'_ Dynamic {
    fn into(self) -> Node {
        match self.type_name() {
            "array" => {
                let a: Array = self.clone().into_array().unwrap();
                let elems: Vec<Node> = a.into_iter().map(|e| e.cast::<Node>()).collect();
                Node::Compound(Compound {
                    type_id: "plain".to_string(),
                    attributes: Default::default(),
                    children: elems,
                })
            }
            _ => Node::Plain(self.to_string()),
        }
    }
}

// pub struct ElementVec(Vec<Element>);
// pub struct ElementChildren(Vec<ElementVec>);

pub fn build_types(engine: &mut Engine) {
    engine.register_type::<Node>();
    // engine.register_type::<ElementChildren>();
}
