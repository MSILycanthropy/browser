mod dom;
mod sink;
pub mod traversal;

pub use html5ever;
use traversal::{Edge, TreeTraversal};

pub use crate::sink::GenericSink;
pub use dom::{DOMNode, DOMTree, SerializableNode};
use html5ever::{
    Parser,
    serialize::{HtmlSerializer, SerializeOpts, Serializer},
    tendril::TendrilSink,
    tokenizer::{BufferQueue, Tokenizer as HtmlTokenizer, TokenizerOpts},
    tree_builder::{TreeBuilder, TreeBuilderOpts},
};
use std::cell::RefCell;
use std::{
    error::Error,
    ops::{Deref, DerefMut},
};

pub fn parse_html_document<T: DOMTree>(dom_tree: T, input: &str) -> T {
    let dom_tree = RefCell::from(dom_tree);
    let sink = GenericSink { dom_tree };

    let tokenizer = Tokenizer::new(sink);

    let parser = Parser::<GenericSink<T>> {
        tokenizer: tokenizer.inner,
        input_buffer: BufferQueue::default(),
    };

    parser.one(input)
}

pub fn serialize_dom_tree<T: DOMTree + TreeTraversal>(
    dom_tree: T,
) -> Result<String, Box<dyn Error>> {
    let options = SerializeOpts::default();
    let buffer = Vec::new();
    let mut serializer = HtmlSerializer::new(buffer, options);

    let root_id = dom_tree.root_id();

    for edge in dom_tree.traverse() {
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

pub(crate) struct Tokenizer<T: DOMTree> {
    pub inner: HtmlTokenizer<TreeBuilder<<<T as DOMTree>::Node as DOMNode>::Id, GenericSink<T>>>,
}

impl<T: DOMTree> Tokenizer<T> {
    pub fn new(sink: GenericSink<T>) -> Self {
        let tree_builder = TreeBuilder::new(sink, TreeBuilderOpts::default());

        Self {
            inner: HtmlTokenizer::new(tree_builder, TokenizerOpts::default()),
        }
    }
}

impl<T: DOMTree> Deref for Tokenizer<T> {
    type Target = HtmlTokenizer<TreeBuilder<<<T as DOMTree>::Node as DOMNode>::Id, GenericSink<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: DOMTree> DerefMut for Tokenizer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
