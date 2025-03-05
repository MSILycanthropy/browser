use std::{collections::HashMap, io::Error};

use ego_tree::{Tree, iter::Edge};
use html5ever::{
    Attribute, QualName,
    interface::QuirksMode,
    serialize::{Serialize, SerializeOpts, Serializer, TraversalScope, serialize},
    tendril::StrTendril,
};
use string_cache::DefaultAtom;

pub use ego_tree::NodeId;

pub type Atom = DefaultAtom;

#[derive(Debug)]
pub struct Document {
    pub tree: Tree<Node>,
    pub quirks_mode: QuirksMode,
}

impl Document {
    pub fn html(&self) -> String {
        let options = SerializeOpts::default();
        let mut buffer = Vec::new();
        serialize(&mut buffer, self, options).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self {
            tree: Tree::new(Node::Document),
            quirks_mode: QuirksMode::NoQuirks,
        }
    }
}

impl Serialize for Document {
    fn serialize<S>(&self, serializer: &mut S, traversal_scope: TraversalScope) -> Result<(), Error>
    where
        S: Serializer,
    {
        let root = self.tree.root();

        for edge in root.traverse() {
            match edge {
                Edge::Open(node) => {
                    if node == root && traversal_scope == TraversalScope::ChildrenOnly(None) {
                        continue;
                    }

                    match node.value() {
                        Node::Doctype { name, .. } => serializer.write_doctype(name)?,
                        Node::Comment(comment) => serializer.write_comment(comment)?,
                        Node::Text(text) => serializer.write_text(text)?,
                        Node::Element(element) => {
                            let attributes =
                                element.attrs.iter().map(|(key, value)| (key, &value[..]));

                            serializer.start_elem(element.name.clone(), attributes)?;
                        }
                        _ => (),
                    }
                }
                Edge::Close(node) => {
                    if node == root && traversal_scope == TraversalScope::ChildrenOnly(None) {
                        continue;
                    }

                    if let Node::Element(element) = node.value() {
                        serializer.end_elem(element.name.clone())?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Node {
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
    Element(ElementData),
}

impl Node {
    pub fn as_element(&self) -> Option<&ElementData> {
        match self {
            Node::Element(data) => Some(data),
            _ => None,
        }
    }

    pub fn as_element_mut(&mut self) -> Option<&mut ElementData> {
        match self {
            Node::Element(data) => Some(data),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ElementData {
    pub name: QualName,
    pub attrs: HashMap<QualName, StrTendril>,
    pub id: Option<DefaultAtom>,
}

impl ElementData {
    pub fn new(name: QualName, raw_attrs: Vec<Attribute>) -> Self {
        let attrs = raw_attrs
            .into_iter()
            .map(|attr| (attr.name, attr.value))
            .collect::<HashMap<_, _>>();
        let id = attrs
            .iter()
            .find(|(name, _)| name.local.as_ref() == "id")
            .map(|(_, value)| Atom::from(value.as_ref()));

        Self { name, id, attrs }
    }
}
