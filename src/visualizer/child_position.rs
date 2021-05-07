use std::num::NonZeroUsize;

/// Whether an item in [`children`](crate::tree::Tree) is the last.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildPosition {
    /// The item is not the last child.
    Init,
    /// The item is the last child.
    Last,
}

impl ChildPosition {
    /// Deduce a child's position from its index and the number of children.
    pub const fn from_index(child_index: usize, child_count: NonZeroUsize) -> Self {
        if child_index + 1 < child_count.get() {
            ChildPosition::Init
        } else {
            ChildPosition::Last
        }
    }
}
