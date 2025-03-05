use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
};

use dom::NodeId;
use html5ever::{
    QualName,
    interface::{ElementFlags, NodeOrText, QuirksMode, TreeSink},
    tendril::StrTendril,
};

use dom::{Document, ElementData, Node};

pub(crate) struct Sink {
    pub document: RefCell<Document>,
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
        self.document.borrow_mut().quirks_mode = mode;
    }

    fn get_document(&self) -> Self::Handle {
        self.document.borrow().tree.root().id()
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        Ref::map(self.document.borrow(), |document| {
            let node = document.tree.get(*target).unwrap().value();

            let element = node
                .as_element()
                .expect("tried to get the name of a non-Element during parsing");

            &element.name
        })
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<html5ever::Attribute>,
        flags: ElementFlags,
    ) -> Self::Handle {
        let element_node = Node::Element(ElementData::new(name, attrs));
        let mut document = self.document.borrow_mut();

        let mut node = document.tree.orphan(element_node);

        if flags.template {
            node.append(Node::DocumentFragment);
        }

        node.id()
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        let comment_node = Node::Comment(text);

        self.document.borrow_mut().tree.orphan(comment_node).id()
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle {
        let pi_node = Node::ProcessingInstruction { target, data };

        self.document.borrow_mut().tree.orphan(pi_node).id()
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let mut document = self.document.borrow_mut();
        let mut parent = document.tree.get_mut(*parent).unwrap();

        match child {
            NodeOrText::AppendNode(id) => {
                parent.append_id(id);
            }
            NodeOrText::AppendText(text) => {
                let did_append = parent.last_child().is_some_and(|mut n| match n.value() {
                    Node::Text(t) => {
                        t.push_tendril(&text);
                        true
                    }
                    _ => false,
                });

                if !did_append {
                    parent.append(Node::Text(text));
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
        let has_parent = self
            .document
            .borrow()
            .tree
            .get(*element)
            .unwrap()
            .parent()
            .is_some();

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
        let doctype_node = Node::Doctype {
            name,
            public_id,
            system_id,
        };

        self.document
            .borrow_mut()
            .tree
            .root_mut()
            .append(doctype_node);
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        self.document
            .borrow()
            .tree
            .get(*target)
            .unwrap()
            .first_child()
            .unwrap()
            .id()
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn append_before_sibling(&self, sibling: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        let mut document = self.document.borrow_mut();
        let mut sibling = document.tree.get_mut(*sibling).unwrap();

        if sibling.parent().is_none() {
            return;
        }

        match new_node {
            NodeOrText::AppendNode(id) => {
                sibling.insert_id_before(id);
            }
            NodeOrText::AppendText(text) => {
                let did_append = sibling.prev_sibling().is_some_and(|mut n| match n.value() {
                    Node::Text(t) => {
                        t.push_tendril(&text);
                        true
                    }
                    _ => false,
                });

                if !did_append {
                    sibling.insert_before(Node::Text(text));
                }
            }
        }
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        let mut document = self.document.borrow_mut();
        let mut node = document.tree.get_mut(*target).unwrap();
        let element = node
            .value()
            .as_element_mut()
            .expect("tried to set attrs on non-Element in HTML parsing");

        for attr in attrs {
            element.attrs.entry(attr.name).or_insert_with(|| attr.value);
        }
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        self.document
            .borrow_mut()
            .tree
            .get_mut(*target)
            .unwrap()
            .detach();
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        self.document
            .borrow_mut()
            .tree
            .get_mut(*new_parent)
            .unwrap()
            .reparent_from_id_append(*node);
    }
}
