use crate::node::{Element, Node, Script};
use anyhow::Result;

pub trait ElementVisitor {
    fn walk_elements(&mut self, elements: &mut [Element]) -> Result<()> {
        elements.iter_mut().try_for_each(|e| self.visit_element(e))
    }

    fn walk_element(&mut self, element: &mut Element) -> Result<()> {
        match element {
            Element::Plain(text) => self.visit_plain(text),
            Element::Node(node) => self.visit_node(node),
            Element::Script(script) => self.visit_script(script),
        }
    }

    fn visit_element(&mut self, element: &mut Element) -> Result<()> {
        self.walk_element(element)
    }

    fn visit_script(&mut self, script: &mut Script) -> Result<()> {
        Ok(())
    }

    fn visit_plain(&mut self, text: &mut String) -> Result<()> {
        Ok(())
    }

    fn visit_node(&mut self, node: &mut Node) -> Result<()> {
        if let Some(children) = &mut node.children {
            self.walk_elements(children)?;
        }

        Ok(())
    }
}
