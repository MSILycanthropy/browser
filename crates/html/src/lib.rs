mod sink;

use crate::sink::Sink;
use dom::{Document, NodeId};
use html5ever::{
    Parser,
    tendril::TendrilSink,
    tokenizer::{BufferQueue, Tokenizer as HtmlTokenizer, TokenizerOpts},
    tree_builder::{TreeBuilder, TreeBuilderOpts},
};
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

pub fn parse_html_document(input: &str) -> Document {
    let document = RefCell::from(Document::new());
    let sink = Sink { document };

    let tokenizer = Tokenizer::new(sink);

    let parser = Parser::<Sink> {
        tokenizer: tokenizer.inner,
        input_buffer: BufferQueue::default(),
    };

    parser.one(input)
}

pub(crate) struct Tokenizer {
    pub inner: HtmlTokenizer<TreeBuilder<NodeId, Sink>>,
}

impl Tokenizer {
    pub fn new(sink: Sink) -> Self {
        let tree_builder = TreeBuilder::new(sink, TreeBuilderOpts::default());

        Self {
            inner: HtmlTokenizer::new(tree_builder, TokenizerOpts::default()),
        }
    }
}

impl Deref for Tokenizer {
    type Target = HtmlTokenizer<TreeBuilder<NodeId, Sink>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Tokenizer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
