use crate::{Document, Node};

pub enum Edge<'dom> {
    Open(&'dom Node),
    Close(&'dom Node),
}

pub(crate) trait TreeTraversal {
    fn traverse(&self) -> Traversal;
}

impl TreeTraversal for Node {
    fn traverse(&self) -> Traversal {
        Traversal::new(self)
    }
}

impl TreeTraversal for Document {
    fn traverse(&self) -> Traversal {
        Traversal::new(self.root())
    }
}

pub(crate) struct Traversal<'dom> {
    stack: Vec<(&'dom Node, bool)>,
}

impl<'dom> Traversal<'dom> {
    fn new(root: &'dom Node) -> Self {
        Self {
            stack: vec![(root, false)],
        }
    }
}

impl<'dom> Iterator for Traversal<'dom> {
    type Item = Edge<'dom>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((node, visited)) = self.stack.pop() {
            if visited {
                return Some(Edge::Close(node));
            }

            self.stack.push((node, true));

            for child_id in node.children.iter().rev() {
                let child = node.lookup(*child_id);

                self.stack.push((child, false));
            }

            return Some(Edge::Open(node));
        }

        None
    }
}
