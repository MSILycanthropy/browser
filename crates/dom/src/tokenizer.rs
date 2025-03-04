use std::ops::{Deref, DerefMut};

use ego_tree::NodeId;
use html5ever::{
    tokenizer::{Tokenizer as HtmlTokenizer, TokenizerOpts},
    tree_builder::{TreeBuilder, TreeBuilderOpts},
};

use crate::sink::Sink;

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
