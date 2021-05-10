/// Whether a node in a tree has children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parenthood {
    /// The node has no children.
    Childless,
    /// The node has children.
    Parent,
}

impl Parenthood {
    /// Deduce parenthood from the number of children.
    pub const fn from_children_count(children_count: usize) -> Self {
        if children_count == 0 {
            Parenthood::Childless
        } else {
            Parenthood::Parent
        }
    }
}
