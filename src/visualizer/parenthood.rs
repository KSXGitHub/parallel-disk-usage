use crate::{size::Size, tree::Tree};

/// Whether a node in a tree has children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parenthood {
    /// The node has no children.
    Childless,
    /// The node has children.
    Parent,
}

impl Parenthood {
    /// Deduce parenthood of a node.
    pub fn from_node<Name, Data: Size>(node: &Tree<Name, Data>) -> Self {
        if node.children().is_empty() {
            Parenthood::Childless
        } else {
            Parenthood::Parent
        }
    }
}
