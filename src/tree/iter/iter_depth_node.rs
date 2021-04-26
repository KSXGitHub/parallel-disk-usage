use super::{PostYield, TraverseIter, Yield};
use crate::{size::Size, tree::Tree};
use derive_more::{AsMut, AsRef, Deref, From, Into};

/// The [`Item`](Iterator::Item) type of `Tree::iter_depth_node`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsMut, AsRef, Deref, From, Into)]
pub struct IterDepthNodeItem<'a, Name, Data: Size> {
    /// Distance from the tree root to this node.
    #[as_mut(ignore)]
    #[as_ref(ignore)]
    pub depth: usize,
    /// The current node.
    #[deref]
    pub node: &'a Tree<Name, Data>,
}

/// The [`Yield`] type of `Tree::iter_depth_node`.
pub struct IterDepthNodeYield;
impl<'a, Name, Data> Yield<'a, Name, Data, usize> for IterDepthNodeYield
where
    Data: Size,
    Tree<Name, Data>: 'a,
{
    type Item = IterDepthNodeItem<'a, Name, Data>;

    fn execute(&mut self, current_depth: &mut usize, node: &'a Tree<Name, Data>) -> Self::Item {
        let depth = *current_depth;
        *current_depth += 1;
        IterDepthNodeItem { depth, node }
    }
}

/// The [`PostYield`] type of `Tree::iter_depth_node`.
pub struct IterDepthNodePostYield;
impl PostYield<usize> for IterDepthNodePostYield {
    fn execute(&mut self, depth: &mut usize) {
        *depth -= 1;
    }
}

/// The return type of `Tree::iter_depth_node`.
pub type IterDepthResult<'a, Name, Data> =
    TraverseIter<'a, Name, Data, usize, IterDepthNodeYield, IterDepthNodePostYield>;

impl<Name, Data: Size> Tree<Name, Data> {
    /// Recursively traverse the tree with depth.
    pub fn iter_depth_node(&self) -> IterDepthResult<'_, Name, Data> {
        self.traverse(IterDepthNodeYield, IterDepthNodePostYield)
    }
}
