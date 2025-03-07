use crate::{Document, Node};

pub enum Edge<'dom> {
    Open(&'dom Node),
    Close(&'dom Node),
}

pub(crate) trait TreeTraversal {
    fn traverse(&self) -> Traversal;
}

impl TreeTraversal for Document {
    fn traverse(&self) -> Traversal {
        Traversal::new(self)
    }
}

pub(crate) struct Traversal<'dom> {
    document: &'dom Document,
    stack: Vec<(&'dom Node, bool)>,
}

impl<'dom> Traversal<'dom> {
    fn new(document: &'dom Document) -> Self {
        let root = document.root();

        Self {
            document,
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
                let child = self.document.node(*child_id)?;

                self.stack.push((child, false));
            }

            return Some(Edge::Open(node));
        }

        None
    }
}
