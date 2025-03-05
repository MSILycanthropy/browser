use dom::{
    Document, Node, NodeId,
    node::{Element, NodeData},
};
use html5ever::{
    QualName,
    interface::{ElementFlags, NodeOrText, QuirksMode, TreeSink},
    tendril::StrTendril,
};
use std::{
    borrow::Cow,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

pub(crate) struct Sink {
    pub(crate) document: RefCell<Document>,
}

impl Sink {
    fn document(&self) -> Ref<Document> {
        self.document.borrow()
    }

    fn document_mut(&self) -> RefMut<Document> {
        self.document.borrow_mut()
    }

    fn node(&self, id: NodeId) -> Ref<Node> {
        Ref::map(self.document(), |document| {
            document.node(id).expect("Node handle is invalid")
        })
    }

    fn node_mut(&self, id: NodeId) -> RefMut<Node> {
        RefMut::map(self.document_mut(), |document| {
            document.node_mut(id).expect("Node handle is invalid")
        })
    }

    fn insert_node(&self, data: NodeData) -> NodeId {
        self.document_mut().insert_node(data)
    }

    fn append_to(&self, parent_id: NodeId, new_child_id: NodeId) {
        self.document_mut().append_to(parent_id, new_child_id);
    }

    fn try_append_text(&self, to_id: Option<NodeId>, text: &StrTendril) -> bool {
        let Some(to_id) = to_id else { return false };
        let mut document = self.document_mut();
        let Some(node) = document.node_mut(to_id) else {
            return false;
        };

        match node.data_mut() {
            NodeData::Text(original) => {
                original.push_tendril(text);
                true
            }
            _ => false,
        }
    }
}

impl TreeSink for Sink {
    type Output = Document;
    type Handle = NodeId;
    type ElemName<'a> = Ref<'a, QualName>;

    fn finish(self) -> Self::Output {
        self.document.into_inner()
    }

    fn parse_error(&self, msg: Cow<'static, str>) {
        println!("Parse error: {}", msg);
    }

    fn set_quirks_mode(&self, mode: QuirksMode) {
        self.document_mut().set_quirks_mode(mode);
    }

    fn get_document(&self) -> Self::Handle {
        self.document().id()
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        Ref::map(self.document.borrow(), |document| {
            let node = document
                .node(*target)
                .expect("Node not found for elem_name");

            let element = node
                .as_element()
                .expect("tried to get the name of a non-Element during parsing");

            element.name()
        })
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<html5ever::Attribute>,
        flags: ElementFlags,
    ) -> Self::Handle {
        let attrs = attrs
            .into_iter()
            .map(|attr| (attr.name, attr.value))
            .collect::<HashMap<_, _>>();

        let id = self.insert_node(NodeData::Element(Element { name, attrs }));

        if flags.template {
            let fragment_id = self.insert_node(NodeData::DocumentFragment);
            self.append_to(id, fragment_id);
        }

        id
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        self.insert_node(NodeData::Text(text))
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle {
        self.insert_node(NodeData::ProcessingInstruction { target, data })
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let parent = *parent;

        match child {
            NodeOrText::AppendNode(id) => {
                self.append_to(parent, id);
            }
            NodeOrText::AppendText(text) => {
                let did_append = self.try_append_text(Some(parent), &text);

                if !did_append {
                    let id = self.insert_node(NodeData::Text(text));

                    self.append_to(parent, id);
                }
            }
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        let has_parent = self.node(*element).parent.is_some();

        if has_parent {
            self.append_before_sibling(element, child);
            return;
        }

        self.append(prev_element, child);
    }

    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    ) {
        let doctype = NodeData::Doctype {
            name,
            public_id,
            system_id,
        };

        let root_id = self.document().id();
        let id = self.insert_node(doctype);
        self.append_to(root_id, id)
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        *self
            .node(*target)
            .children
            .first()
            .expect("Template node has no children")
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn append_before_sibling(&self, sibling_id: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        let sibling = self.node(*sibling_id);
        let parent_id = sibling.parent.expect("Sibling has no parent");
        let mut parent = self.node_mut(parent_id);

        let sibling_index = parent
            .children
            .iter()
            .position(|child_id| child_id == sibling_id)
            .expect("Sibling not within parent");

        let child_id = match new_node {
            NodeOrText::AppendText(text) => {
                let prev_sibling_id = match sibling_index {
                    0 => None,
                    i => Some(parent.children[i - 1]),
                };

                let did_append = self.try_append_text(prev_sibling_id, &text);

                if did_append {
                    return;
                } else {
                    self.insert_node(NodeData::Text(text))
                }
            }
            NodeOrText::AppendNode(id) => id,
        };

        let mut prev_sibling = self.node_mut(child_id);
        prev_sibling.parent = Some(parent_id);
        parent.children.insert(sibling_index, child_id)
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        let mut node = self.node_mut(*target);
        let element = node
            .as_element_mut()
            .expect("tried to set attrs on non-Element in HTML parsing");

        for attr in attrs {
            element.attrs.entry(attr.name).or_insert_with(|| attr.value);
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        let mut node = self.node_mut(*target);
        let parent = node.parent.take().expect("Node has no parent");

        self.node_mut(parent)
            .children
            .retain(|child_id| child_id != target);
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        let new_parent = *new_parent;
        let children = std::mem::take(&mut self.node_mut(*node).children);

        for child in children.iter() {
            self.node_mut(*child).parent = Some(new_parent)
        }

        self.node_mut(new_parent).children.extend(&children);
    }
}
