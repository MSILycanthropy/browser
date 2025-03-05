use std::io::Error;

use html5ever::{
    interface::QuirksMode::{self, NoQuirks},
    serialize::{Serialize, SerializeOpts, Serializer, TraversalScope, serialize},
};

use crate::{
    node::{Node, NodeArena, NodeData, NodeId},
    traversal::{Edge, TreeTraversal},
};

pub struct Document {
    id: Option<NodeId>,

    nodes: Box<NodeArena>,

    quirks_mode: QuirksMode,
}

impl Document {
    pub fn new() -> Self {
        let nodes = Box::new(NodeArena::with_capacity_and_key(1));
        let mut instance = Self {
            id: None,
            nodes,
            quirks_mode: NoQuirks,
        };
        let id = instance.insert_node(NodeData::Document);
        instance.id = Some(id);
        instance
    }

    pub fn html(&self) -> String {
        let options = SerializeOpts::default();
        let mut buffer = Vec::new();
        serialize(&mut buffer, self, options).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    pub fn insert_node(&mut self, node_data: NodeData) -> NodeId {
        let tree = self.nodes.as_mut() as *mut NodeArena;
        self.nodes
            .insert_with_key(|key| Node::new(key, node_data, tree))
    }

    pub fn id(&self) -> NodeId {
        match self.id {
            Some(id) => id,
            None => unreachable!(),
        }
    }

    pub fn tree(&self) -> &NodeArena {
        self.nodes.as_ref()
    }

    pub fn tree_mut(&mut self) -> &mut NodeArena {
        self.nodes.as_mut()
    }

    pub fn quirks_mode(&self) -> QuirksMode {
        self.quirks_mode
    }

    pub fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        self.quirks_mode = quirks_mode;
    }

    pub fn root(&self) -> &Node {
        self.node(self.id()).expect("Document has no root node")
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    pub fn append_to(&mut self, parent_id: NodeId, new_child_id: NodeId) {
        let node = self
            .nodes
            .get_mut(parent_id)
            .expect("Parent is not in the DOM");

        node.children.push(new_child_id);
    }
}

impl Serialize for Document {
    fn serialize<S>(&self, serializer: &mut S, traversal_scope: TraversalScope) -> Result<(), Error>
    where
        S: Serializer,
    {
        let root = self.root();

        for edge in root.traverse() {
            match edge {
                Edge::Open(node) => {
                    if node.id == root.id && traversal_scope == TraversalScope::ChildrenOnly(None) {
                        continue;
                    }

                    match node.data() {
                        NodeData::Doctype { name, .. } => serializer.write_doctype(name)?,
                        NodeData::Comment(comment) => serializer.write_comment(comment)?,
                        NodeData::Text(text) => serializer.write_text(text)?,
                        NodeData::Element(element) => {
                            let attributes =
                                element.attrs.iter().map(|(key, value)| (key, &value[..]));

                            serializer.start_elem(element.name.clone(), attributes)?;
                        }
                        _ => (),
                    }
                }
                Edge::Close(node) => {
                    if node.id == root.id && traversal_scope == TraversalScope::ChildrenOnly(None) {
                        continue;
                    }

                    if let NodeData::Element(element) = node.data() {
                        serializer.end_elem(element.name.clone())?;
                    }
                }
            }
        }

        Ok(())
    }
}
