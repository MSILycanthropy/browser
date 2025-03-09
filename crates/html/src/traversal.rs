use crate::{DOMNode, DOMTree};

pub enum Edge<'dom, T: DOMTree> {
    Open(&'dom <T as DOMTree>::Node),
    Close(&'dom <T as DOMTree>::Node),
}

pub trait TreeTraversal
where
    Self: DOMTree + Sized,
{
    fn traverse(&self) -> Traversal<Self> {
        Traversal::new(self)
    }
}

pub struct Traversal<'dom, T: DOMTree> {
    dom_tree: &'dom T,
    stack: Vec<(&'dom <T as DOMTree>::Node, bool)>,
}

impl<'dom, T: DOMTree> Traversal<'dom, T> {
    pub fn new(dom_tree: &'dom T) -> Self {
        let root = dom_tree.root();

        Self {
            dom_tree,
            stack: vec![(root, false)],
        }
    }
}

impl<'dom, T: DOMTree> Iterator for Traversal<'dom, T> {
    type Item = Edge<'dom, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((node, visited)) = self.stack.pop() {
            if visited {
                return Some(Edge::Close(node));
            }

            self.stack.push((node, true));

            for child_id in node.children().iter().rev() {
                let child = self.dom_tree.node(*child_id)?;

                self.stack.push((child, false));
            }

            return Some(Edge::Open(node));
        }

        None
    }
}
