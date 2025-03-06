use std::collections::HashMap;

use html5ever::{QualName, tendril::StrTendril};

use slotmap::{SlotMap, new_key_type};

new_key_type! { pub struct NodeId; }

pub type NodeArena = SlotMap<NodeId, Node>;

#[derive(PartialEq, Debug)]
pub struct Node {
    pub id: NodeId,

    tree: *mut NodeArena,

    data: NodeData,

    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

impl Node {
    pub fn new(id: NodeId, data: NodeData, tree: *mut NodeArena) -> Self {
        Self {
            id,
            data,
            tree,
            children: vec![],
            parent: None,
        }
    }

    pub fn tree(&self) -> &NodeArena {
        unsafe { &*self.tree }
    }

    pub fn lookup(&self, id: NodeId) -> &Self {
        self.tree().get(id).expect("Node does not exist in tree")
    }

    pub fn data(&self) -> &NodeData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut NodeData {
        &mut self.data
    }

    pub fn as_element(&self) -> Option<&Element> {
        match self.data() {
            NodeData::Element(element) => Some(element),
            _ => None,
        }
    }

    pub fn as_element_mut(&mut self) -> Option<&mut Element> {
        match self.data_mut() {
            NodeData::Element(element) => Some(element),
            _ => None,
        }
    }

    pub fn index_as_child(&self) -> usize {
        self.parent
            .map(|p| self.lookup(p))
            .and_then(|node| node.children.iter().position(|c| c == &self.id))
            .unwrap_or(0)
    }

    pub fn nth_prev_sibling_id(&self, n: usize) -> Option<&NodeId> {
        let parent_id = self.parent?;
        let parent = self.lookup(parent_id);

        parent.children.get(self.index_as_child() - n)
    }

    pub fn nth_next_sibling_id(&self, n: usize) -> Option<&NodeId> {
        let parent_id = self.parent?;
        let parent = self.lookup(parent_id);

        parent.children.get(self.index_as_child() + n)
    }
}

#[derive(PartialEq, Debug)]
pub enum NodeData {
    Document,
    DocumentFragment,
    Doctype {
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    },
    Text(StrTendril),
    Comment(StrTendril),
    ProcessingInstruction {
        target: StrTendril,
        data: StrTendril,
    },
    Element(Element),
}

#[derive(PartialEq, Debug)]
pub struct Element {
    pub name: QualName,
    pub attrs: HashMap<QualName, StrTendril>,
}

impl Element {
    pub fn name(&self) -> &QualName {
        &self.name
    }
}
