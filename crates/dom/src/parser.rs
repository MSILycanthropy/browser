use std::cell::RefCell;

use html5ever::{Parser, tendril::TendrilSink, tokenizer::BufferQueue};

use crate::{Document, sink::Sink, tokenizer::Tokenizer};

pub fn parse_html_document(input: &str) -> Document {
    let document = RefCell::from(Document::default());
    let sink = Sink { document };

    let tokenizer = Tokenizer::new(sink);

    let parser = Parser::<Sink> {
        tokenizer: tokenizer.inner,
        input_buffer: BufferQueue::default(),
    };

    parser.one(input)
}
