use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use html5ever::{QualName, tendril::StrTendril};

use slotmap::{SlotMap, new_key_type};

new_key_type! { pub struct NodeId; }

pub type NodeArena = SlotMap<NodeId, Node>;

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
}

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

pub struct Element {
    pub name: QualName,
    pub attrs: HashMap<QualName, StrTendril>,
}

impl Element {
    pub fn name(&self) -> &QualName {
        &self.name
    }
}
