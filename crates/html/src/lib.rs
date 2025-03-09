mod dom;
mod sink;

pub use html5ever;

pub use crate::sink::GenericSink;
pub use dom::{DOMNode, DOMTree};
use html5ever::{
    Parser,
    tendril::TendrilSink,
    tokenizer::{BufferQueue, Tokenizer as HtmlTokenizer, TokenizerOpts},
    tree_builder::{TreeBuilder, TreeBuilderOpts},
};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

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
