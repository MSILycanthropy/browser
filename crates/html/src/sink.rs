use std::{
    borrow::Cow,
    cell::{Ref, RefCell, RefMut},
};

use html5ever::{
    Attribute, QualName,
    interface::{ElementFlags, NodeOrText, QuirksMode, TreeSink},
    tendril::StrTendril,
};

use crate::dom::{DOMNode, DOMTree};

pub struct GenericSink<T: DOMTree> {
    pub dom_tree: RefCell<T>,
}

impl<T: DOMTree> GenericSink<T> {
    pub fn new(dom_tree: T) -> Self {
        Self {
            dom_tree: RefCell::new(dom_tree),
        }
    }
}

impl<T: DOMTree> GenericSink<T> {
    pub(crate) fn dom_tree(&self) -> Ref<T> {
        self.dom_tree.borrow()
    }

    pub(crate) fn dom_tree_mut(&self) -> RefMut<T> {
        self.dom_tree.borrow_mut()
    }

    pub(crate) fn node(&self, id: <<T as DOMTree>::Node as DOMNode>::Id) -> Ref<T::Node> {
        Ref::map(self.dom_tree(), |d| {
            d.node(id)
                .expect("Id does not reference a node in the DOMTree")
        })
    }

    pub(crate) fn node_mut(&self, id: <<T as DOMTree>::Node as DOMNode>::Id) -> RefMut<T::Node> {
        RefMut::map(self.dom_tree_mut(), |d| {
            d.node_mut(id)
                .expect("Id does not reference a node in the DOMTree")
        })
    }

    pub(crate) fn try_append_to_text_node(
        &self,
        to_id: Option<<<T as DOMTree>::Node as DOMNode>::Id>,
        text: &StrTendril,
    ) -> bool {
        let Some(to_id) = to_id else { return false };
        dbg!("try append");
        let mut to = self.node_mut(to_id);
        dbg!("after try append");

        to.try_append_to_text_node(text)
    }
}

impl<T: DOMTree> TreeSink for GenericSink<T> {
    type Output = T;
    type Handle = <<T as DOMTree>::Node as DOMNode>::Id;
    type ElemName<'a>
        = Ref<'a, QualName>
    where
        T: 'a;

    fn finish(self) -> Self::Output {
        self.dom_tree.into_inner()
    }

    fn parse_error(&self, msg: Cow<'static, str>) {
        self.dom_tree().handle_parser_error(msg);
    }

    fn set_quirks_mode(&self, quirks_mode: QuirksMode) {
        self.dom_tree_mut().set_quirks_mode(quirks_mode);
    }

    fn get_document(&self) -> Self::Handle {
        self.dom_tree().root_id()
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> Self::ElemName<'a> {
        Ref::map(self.dom_tree(), |d| {
            d.node(*target)
                .expect("Invalid node reference")
                .element_name()
                .expect("Invalid node reference")
        })
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> Self::Handle {
        let element_id = self.dom_tree_mut().create_element(name, attrs);

        // TODO: Might be better to check the name and see if its `template`?
        if flags.template {
            let fragment_id = self.dom_tree_mut().create_fragment();

            self.append(&element_id, NodeOrText::AppendNode(fragment_id));
        }

        element_id
    }

    fn create_comment(&self, text: StrTendril) -> Self::Handle {
        self.dom_tree_mut().create_comment(text)
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> Self::Handle {
        self.dom_tree_mut()
            .create_processing_instruction(target, data)
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        let parent_id = *parent;

        match child {
            NodeOrText::AppendNode(child_id) => {
                self.dom_tree_mut()
                    .node_mut(parent_id)
                    .unwrap()
                    .append_child(child_id);
            }
            NodeOrText::AppendText(text) => {
                let did_append = self.try_append_to_text_node(Some(parent_id), &text);

                if !did_append {
                    let child_id = self.dom_tree_mut().create_text_node(text);

                    self.dom_tree_mut()
                        .node_mut(parent_id)
                        .unwrap()
                        .append_child(child_id);
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
        let has_parent = self.node(*element).parent().is_some();

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
        let doctype_id = self
            .dom_tree_mut()
            .create_doctype(name, public_id, system_id);

        let root = self.dom_tree().root_id();
        self.append(&root, NodeOrText::AppendNode(doctype_id));
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        *self
            .node(*target)
            .children()
            .first()
            .expect("Template has no children")
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn append_before_sibling(&self, sibling_id: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        let sibling = self.node(*sibling_id);
        let parent_id = sibling.parent().expect("Sibling has no parent");
        let mut parent = self.node_mut(parent_id);

        let sibling_index = parent
            .children()
            .iter()
            .position(|cid| cid == sibling_id)
            .expect("Sibling not within parent");

        let child_id = match new_node {
            NodeOrText::AppendText(text) => {
                let prev_sibling_id = match sibling_index {
                    0 => None,
                    i => Some(parent.children()[i - 1]),
                };

                let did_append = self.try_append_to_text_node(prev_sibling_id, &text);

                if did_append {
                    return;
                }

                self.dom_tree_mut().create_text_node(text)
            }
            NodeOrText::AppendNode(id) => id,
        };

        let mut prev_sibling = self.node_mut(child_id);
        prev_sibling.reparent(parent_id);
        parent.insert_child(sibling_index, child_id);
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {
        let mut node = self.node_mut(*target);

        node.try_merge_attrs(attrs);
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        let mut node = self.node_mut(*target);
        let Some(parent_id) = node.parent() else {
            return;
        };

        let mut parent_node = self.node_mut(parent_id);

        node.detach_from_parent();
        parent_node.remove_child(*target);
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent_id: &Self::Handle) {
        let node = self.node(*node);
        let children = std::mem::take(&mut node.children());

        for child in children.iter() {
            self.node_mut(*child).reparent(*new_parent_id);
        }

        let mut new_parent = self.node_mut(*new_parent_id);
        new_parent.append_children(children);
    }
}
