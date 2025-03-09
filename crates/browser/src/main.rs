use std::collections::HashSet;

use html::{
    DOMNode, DOMTree,
    html5ever::{Attribute, QualName, interface::QuirksMode, tendril::StrTendril},
    parse_html_document,
};

const HTML: &str = r#"
<!DOCTYPE html>
<html>
<body>
<h1>My First Heading</h1>
<p>My first paragraph.</p>
</body>
</html>
"#;

#[derive(Debug)]
struct Document {
    id: usize,

    nodes: Vec<Node>,

    quirks_mode: QuirksMode,
}

impl Document {
    fn new() -> Self {
        let mut instance = Self {
            id: 1,
            nodes: vec![],
            quirks_mode: QuirksMode::NoQuirks,
        };

        instance.insert_node(NodeData::Document);

        return instance;
    }

    fn insert_node(&mut self, data: NodeData) -> usize {
        let next_index = self.nodes.len() + 1;

        self.nodes.push(Node::new(data));

        next_index
    }
}

impl DOMTree for Document {
    type Node = Node;

    fn handle_parser_error(&self, msg: std::borrow::Cow<'static, str>) {
        eprintln!("{}", msg);
    }

    fn root_id(&self) -> usize {
        self.id
    }

    fn node(&self, id: usize) -> Option<&Node> {
        self.nodes.get(id - 1)
    }

    fn node_mut(&mut self, id: usize) -> Option<&mut Node> {
        self.nodes.get_mut(id - 1)
    }

    fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        self.quirks_mode = quirks_mode;
    }

    fn create_comment(&mut self, data: StrTendril) -> usize {
        self.insert_node(NodeData::Comment(data))
    }

    fn create_doctype(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) -> usize {
        self.insert_node(NodeData::Doctype {
            name,
            public_id,
            system_id,
        })
    }

    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>) -> usize {
        self.insert_node(NodeData::Element { name, attrs })
    }

    fn create_fragment(&mut self) -> usize {
        self.insert_node(NodeData::DocumentFragment)
    }

    fn create_processing_instruction(&mut self, target: StrTendril, data: StrTendril) -> usize {
        self.insert_node(NodeData::ProcessingInstruction { target, data })
    }

    fn create_text_node(&mut self, data: StrTendril) -> usize {
        self.insert_node(NodeData::Text(data))
    }
}

#[derive(Debug)]
struct Node {
    data: NodeData,

    parent: Option<usize>,
    children: Vec<usize>,
}

impl Node {
    fn new(data: NodeData) -> Self {
        Self {
            data,
            parent: None,
            children: vec![],
        }
    }
}

impl DOMNode for Node {
    type Id = usize;

    fn append_child(&mut self, id: Self::Id) {
        self.children.push(id);
    }

    fn append_children(&mut self, ids: &[Self::Id]) {
        self.children.extend(ids);
    }

    fn children(&self) -> &[Self::Id] {
        self.children.as_slice()
    }

    fn detach_from_parent(&mut self) {
        self.parent = None
    }

    fn insert_child(&mut self, index: usize, id: Self::Id) {
        self.children.insert(index, id);
    }

    fn remove_child(&mut self, id: Self::Id) {
        let Some(index) = self.children.iter().position(|cid| *cid == id) else {
            return;
        };

        self.children.remove(index);
    }

    fn element_name(&self) -> Option<&QualName> {
        match &self.data {
            NodeData::Element { name, .. } => Some(name),
            _ => None,
        }
    }

    fn parent(&self) -> Option<Self::Id> {
        self.parent
    }

    fn reparent(&mut self, parent: Self::Id) {
        self.parent = Some(parent);
    }

    fn try_append_to_text_node(&mut self, str: &StrTendril) -> bool {
        match &mut self.data {
            NodeData::Text(text) => {
                text.push_tendril(str);

                true
            }
            _ => false,
        }
    }

    fn try_merge_attrs(&mut self, new_attrs: Vec<Attribute>) -> bool {
        match &mut self.data {
            NodeData::Element { attrs, .. } => {
                let existing_names: HashSet<QualName> =
                    attrs.iter().map(|attr| attr.name.clone()).collect();

                attrs.extend(
                    new_attrs
                        .into_iter()
                        .filter(|attr| !existing_names.contains(&attr.name)),
                );

                true
            }
            _ => false,
        }
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
    Element {
        name: QualName,
        attrs: Vec<Attribute>,
    },
}

fn main() {
    let document = Document::new();
    let document = parse_html_document(document, HTML);

    dbg!(document);
}
