use std::collections::LinkedList;
use std::rc::Rc;

#[derive(Clone)]
pub struct Node<D: Clone> {
    content: Rc<D>,
    children: Vec<Node<D>>,
}

impl<D: Clone> Node<D> {
    pub fn new(content: D, children: Vec<Node<D>>) -> Self {
        Node {
            content: Rc::new(content),
            children,
        }
    }
}

pub struct Config<D: Clone> {
    root: Node<D>,
}

pub struct NodeIterator<D: Clone> {
    node_heap: LinkedList<Node<D>>,
    pos: Vec<usize>,
    layer_len: Vec<usize>,
    current_layer: usize,
}

#[derive(Debug)]
pub struct NodeItem<D> {
    inner: Rc<D>,
    position: Vec<usize>,
}

impl<D: Clone> NodeIterator<D> {
    pub fn new(root: Node<D>) -> NodeIterator<D> {
        NodeIterator {
            node_heap: LinkedList::from([root]),
            pos: vec![0],
            layer_len: vec![0, 0],
            current_layer: 0,
        }
    }
}

impl<D: Clone> IntoIterator for Node<D> {
    type Item = NodeItem<D>;
    type IntoIter = NodeIterator<D>;

    fn into_iter(self) -> Self::IntoIter {
        NodeIterator::new(self)
    }
}

impl<D: Clone> Iterator for NodeIterator<D> {
    type Item = NodeItem<D>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.node_heap.pop_back() {
            None => None,
            Some(mut node) => {
                node.children
                    .iter()
                    .for_each(|c| self.node_heap.push_front(c.clone()));

                let current_pos = self.pos.clone();
                *self.layer_len.last_mut().unwrap() += node.children.len();

                if self.pos.last().unwrap() + 1 >= self.layer_len[self.current_layer] {
                    self.pos.push(0);
                    self.layer_len.push(0);
                    self.current_layer += 1;
                } else {
                    *self.pos.last_mut().unwrap() += 1;
                }

                // node.children.into_iter().for_each(|c| self.node_heap.push(c));
                Some(NodeItem {
                    inner: node.content.clone(),
                    position: current_pos,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node() {
        let config = Node::new(
            1,
            vec![
                Node::new(
                    2,
                    vec![
                        Node::new(4, vec![]),
                        Node::new(5, vec![]),
                        Node::new(5, vec![]),
                        Node::new(5, vec![]),
                    ],
                ),
                Node::new(
                    3,
                    vec![
                        Node::new(6, vec![]),
                        Node::new(5, vec![Node::new(5, vec![]), Node::new(5, vec![])]),
                    ],
                ),
            ],
        );

        let it = config.into_iter();
        let linear: Vec<NodeItem<i32>> = it.collect();
        println!("{:?}", linear);
    }
}
