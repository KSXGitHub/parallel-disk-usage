use super::{PostYield, TraverseIter, Yield};
use crate::{size::Size, tree::Tree};
use derive_more::{AsMut, AsRef, Deref, From, Into};

/// The [`Item`](Iterator::Item) type of `Tree::iter_depth_index_node`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsMut, AsRef, Deref, From, Into)]
pub struct IterDepthIndexNodeItem<'a, Name, Data: Size> {
    /// Distance from the tree root to this node.
    #[as_mut(ignore)]
    #[as_ref(ignore)]
    pub depth: usize,
    /// Index of this node amongst its peer.
    #[as_mut(ignore)]
    #[as_ref(ignore)]
    pub index: usize,
    /// The current node.
    #[deref]
    pub node: &'a Tree<Name, Data>,
}

/// The [`Yield`] type of `Tree::iter_depth_index_node`.
pub struct IterDepthIndexNodeYield;
impl<'a, Name, Data: Size> Yield<'a, Name, Data, (Vec<usize>, usize)> for IterDepthIndexNodeYield
where
    Tree<Name, Data>: 'a,
{
    type Item = IterDepthIndexNodeItem<'a, Name, Data>;

    fn execute(
        &mut self,
        (ancestor_indices, current_index): &mut (Vec<usize>, usize),
        node: &'a Tree<Name, Data>,
    ) -> Self::Item {
        let depth = ancestor_indices.len();
        let index = *current_index;
        ancestor_indices.push(index + 1);
        *current_index = 0;
        IterDepthIndexNodeItem { depth, index, node }
    }
}

/// The [`PostYield`] type of `Tree::iter_depth_index_node`.
pub struct IterDepthIndexNodePostYield;
impl PostYield<(Vec<usize>, usize)> for IterDepthIndexNodePostYield {
    fn execute(&mut self, (ancestor_indices, current_index): &mut (Vec<usize>, usize)) {
        *current_index = ancestor_indices
            .pop()
            .expect("ancestor_indices is never empty when PostYield::execute is called")
    }
}

/// The return type of `Tree::iter_depth_index_node`.
pub type IterDepthIndexNodeResult<'a, Name, Data> = TraverseIter<
    'a,
    Name,
    Data,
    (Vec<usize>, usize),
    IterDepthIndexNodeYield,
    IterDepthIndexNodePostYield,
>;

impl<Name, Data: Size> Tree<Name, Data> {
    pub fn iter_depth_index_node(&self) -> IterDepthIndexNodeResult<'_, Name, Data> {
        self.traverse(IterDepthIndexNodeYield, IterDepthIndexNodePostYield)
    }
}
