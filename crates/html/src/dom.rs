use html5ever::{Attribute, QualName, interface::QuirksMode, tendril::StrTendril};
use std::borrow::Cow;

pub trait DOMTree: std::fmt::Debug {
    type Node: DOMNode;

    fn handle_parser_error(&self, msg: Cow<'static, str>);

    fn set_quirks_mode(&mut self, quirks_mode: QuirksMode);

    fn root_id(&self) -> <<Self as DOMTree>::Node as DOMNode>::Id;

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
}

pub trait DOMNode {
    type Id: Clone + Copy + Eq + std::fmt::Debug;

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
}
