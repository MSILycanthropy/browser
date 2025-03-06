use dom::{node::Element, Document};

mod node;

// TODO: iirc TDocument must impl Copy, but we also need access to a
// SharedRwLock, which _cant_ be Copy. So.. where should we put it?
pub struct LayoutDocument<'dom> {
  inner: &'dom Document
}
pub struct LayoutElement<'dom> {
  inner: &'dom Element
}

// TODO: No idea how this will actually need to work.. so.. shrugging emoji
pub struct LayoutShadowRoot;
