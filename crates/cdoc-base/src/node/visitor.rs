use crate::node::{Compound, Node, Script};
use anyhow::Result;

pub trait NodeVisitor {
    fn walk_elements(&mut self, elements: &mut [Node]) -> Result<()> {
        elements.iter_mut().try_for_each(|e| self.visit_element(e))
    }

    fn walk_element(&mut self, element: &mut Node) -> Result<()> {
        match element {
            Node::Plain(text) => self.visit_plain(text),
            Node::Compound(node) => self.visit_compound(node),
            Node::Script(script) => self.visit_script(script),
        }
    }

    fn walk_node(&mut self, node: &mut Compound) -> Result<()> {
        self.walk_elements(&mut node.children)
    }

    fn visit_element(&mut self, element: &mut Node) -> Result<()> {
        self.walk_element(element)
    }

    fn visit_script(&mut self, script: &mut Script) -> Result<()> {
        Ok(())
    }

    fn visit_plain(&mut self, text: &mut String) -> Result<()> {
        Ok(())
    }

    fn visit_compound(&mut self, node: &mut Compound) -> Result<()> {
        self.walk_node(node)
    }
}