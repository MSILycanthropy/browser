use std::ops::Deref;

use dom::{node::NodeData, Node};
use style::dom::{NodeInfo, TNode};

use crate::LayoutDocument;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct LayoutNode<'dom> {
  inner: &'dom Node
}

impl<'dom> LayoutNode<'dom> {
  fn parent(&self) -> Option<LayoutNode<'dom>> {
    let parent = self.parent.map(|id| self.lookup(id))?;

    Some(parent.into())
  }
}

impl<'dom> From<&'dom Node> for LayoutNode<'dom> {
  fn from(value: &'dom Node) -> Self {
      Self { inner: value }
  }
}

impl<'dom> Deref for LayoutNode<'dom> {
  type Target = &'dom Node;

  fn deref(&self) -> &Self::Target {
      &self.inner
  }
}

impl<'dom> NodeInfo for LayoutNode<'dom> {
  fn is_element(&self) -> bool {
    self.as_element().is_some()
  }

  fn is_text_node(&self) -> bool {
    match self.data() {
      NodeData::Text(_) => true,
      _ => false
    }
  }
}


impl<'dom> TNode for LayoutNode<'dom> {
  // TODO: impl TDocument
  type ConcreteDocument = LayoutNode<'dom>;

  // TODO: These need to maybe not be nodes?
  type ConcreteElement = LayoutNode<'dom>; 
  type ConcreteShadowRoot = LayoutNode<'dom>;
  
  fn parent_element(&self) -> Option<Self::ConcreteElement> {
    self.parent()
  }

  fn first_child(&self) -> Option<Self> {
      let child_id = self.children.first()?;
      let child_node = self.lookup(*child_id);

      Some(child_node.into())
  }

  fn last_child(&self) -> Option<Self> {
      let child_id = self.children.last()?;
      let child_node = self.lookup(*child_id);

      Some(child_node.into())
  }

  fn prev_sibling(&self) -> Option<Self> {
      let sibling_id = self.nth_prev_sibling_id(1)?;
      let sibling_node = self.lookup(*sibling_id);

      Some(sibling_node.into())
  }

  fn next_sibling(&self) -> Option<Self> {
      let sibling_id = self.nth_next_sibling_id(1)?;
      let sibling_node = self.lookup(*sibling_id);

      Some(sibling_node.into())
  }

  fn owner_doc(&self) -> Self::ConcreteDocument {
      todo!("Refactor `Node` to just have a reference to `Document`")
  }

  fn is_in_document(&self) -> bool {
      true
  }

  // TODO: in servo this does something a bit fancier.. so we should probably..
  // also do that?
  fn traversal_parent(&self) -> Option<Self::ConcreteElement> {
      self.parent()
  }

  fn opaque(&self) -> style::dom::OpaqueNode {
      todo!("I don't know what this is yet")
  }

  fn debug_id(self) -> usize {
      todo!("Refactor from Slotmap to Slab.. or just use usize as key?")
  }

  fn as_element(&self) -> Option<Self::ConcreteElement> {
      match self.data() {
        NodeData::Element(_) => Some(*self),
        _ => None
      }
  }

  // TODO: This might break if we impl TDocument for LayoutDocument
  fn as_document(&self) -> Option<Self::ConcreteDocument> {
      match self.data() {
        NodeData::Document => Some(*self),
        _ => None
      }
  }

  
}
