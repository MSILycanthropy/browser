use html5ever::{
    Attribute, QualName,
    interface::QuirksMode,
    serialize::{HtmlSerializer, SerializeOpts, Serializer},
    tendril::{StrTendril, TendrilSink},
};
use std::{borrow::Cow, cell::RefCell, error::Error};

use crate::{
    GenericSink,
    traversal::{Edge, TreeTraversal},
};

#[derive(Default)]
pub struct ParserOpts {
    fragment: Option<(QualName, Vec<Attribute>)>,
}

pub trait DOMTree: Default + std::fmt::Debug {
    type Node: DOMNode;

    fn handle_parser_error(&self, msg: Cow<'static, str>);

    fn set_quirks_mode(&mut self, quirks_mode: QuirksMode);

    fn root_id(&self) -> <<Self as DOMTree>::Node as DOMNode>::Id;

    fn root(&self) -> &Self::Node {
        let id = self.root_id();

        self.node(id).expect("No root node found")
    }

    fn node(&self, id: <<Self as DOMTree>::Node as DOMNode>::Id) -> Option<&Self::Node>;

    fn node_mut(&mut self, id: <<Self as DOMTree>::Node as DOMNode>::Id)
    -> Option<&mut Self::Node>;

    fn create_fragment(&mut self) -> <<Self as DOMTree>::Node as DOMNode>::Id;

    fn create_doctype(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) -> <<Self as DOMTree>::Node as DOMNode>::Id;

    fn create_text_node(&mut self, data: StrTendril) -> <<Self as DOMTree>::Node as DOMNode>::Id;

    fn create_comment(&mut self, data: StrTendril) -> <<Self as DOMTree>::Node as DOMNode>::Id;

    fn create_processing_instruction(
        &mut self,
        target: StrTendril,
        data: StrTendril,
    ) -> <<Self as DOMTree>::Node as DOMNode>::Id;

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
    ) -> <<Self as DOMTree>::Node as DOMNode>::Id;

    fn parse(input: &str, opts: ParserOpts) -> Self {
        Self::parse_with(|| Self::default(), input, opts)
    }

    fn parse_with<F>(factory: F, input: &str, opts: ParserOpts) -> Self
    where
        F: FnOnce() -> Self,
    {
        let instance = factory();
        let dom_tree = RefCell::from(instance);
        let sink = GenericSink { dom_tree };

        let parser = match opts.fragment {
            Some((name, attrs)) => {
                html5ever::driver::parse_fragment(sink, Default::default(), name, attrs)
            }
            None => html5ever::driver::parse_document(sink, Default::default()),
        };

        parser.one(input)
    }

    fn serialize(&self) -> Result<String, Box<dyn Error>>
    where
        Self: TreeTraversal,
    {
        let options = SerializeOpts::default();
        let buffer = Vec::new();
        let mut serializer = HtmlSerializer::new(buffer, options);

        let root_id = self.root_id();

        for edge in self.traverse() {
            match edge {
                Edge::Open(node) => {
                    if node.id() == root_id {
                        continue;
                    }

                    match node.serializable_data() {
                        SerializableNode::Doctype(name) => serializer.write_doctype(name)?,
                        SerializableNode::Comment(comment) => serializer.write_comment(comment)?,
                        SerializableNode::Text(text) => serializer.write_text(text)?,
                        SerializableNode::Element(name, attrs) => {
                            let attributes = attrs.iter().map(|attr| (&attr.name, &attr.value[..]));

                            serializer.start_elem(name.clone(), attributes)?;
                        }
                        _ => (),
                    }
                }
                Edge::Close(node) => {
                    if node.id() == root_id {
                        continue;
                    }

                    if let SerializableNode::Element(name, ..) = node.serializable_data() {
                        serializer.end_elem(name.clone())?;
                    }
                }
            }
        }

        let str = String::from_utf8(serializer.writer)?;

        Ok(str)
    }
}

pub enum SerializableNode<'dom> {
    Doctype(&'dom StrTendril),
    Comment(&'dom StrTendril),
    Text(&'dom StrTendril),
    Element(&'dom QualName, &'dom Vec<Attribute>),
    None,
}

pub trait DOMNode {
    type Id: Clone + Copy + Eq + std::fmt::Debug;

    fn id(&self) -> Self::Id;

    fn parent(&self) -> Option<Self::Id>;

    fn reparent(&mut self, parent: Self::Id);

    fn detach_from_parent(&mut self);

    fn children(&self) -> &[Self::Id];

    fn append_child(&mut self, id: Self::Id);

    fn append_children(&mut self, ids: &[Self::Id]);

    fn insert_child(&mut self, index: usize, id: Self::Id);

    fn remove_child(&mut self, id: Self::Id);

    // If this dom node is a Text node, append to it and return true
    // otherwise return false
    fn try_append_to_text_node(&mut self, str: &StrTendril) -> bool;

    // If this dom node is an Element node, merge it's attributes with
    // the attrs passed in and return true. Else return false.
    fn try_merge_attrs(&mut self, attrs: Vec<Attribute>) -> bool;

    fn element_name(&self) -> Option<&QualName>;

    fn serializable_data(&self) -> SerializableNode;
}
