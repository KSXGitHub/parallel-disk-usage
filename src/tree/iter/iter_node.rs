use super::{PostYield, TraverseIter, Yield};
use crate::{size::Size, tree::Tree};

/// The [`Yield`] type of `Tree::iter_tree`.
pub struct IterTreeYield;
impl<'a, Name, Data: Size> Yield<'a, Name, Data, ()> for IterTreeYield
where
    Tree<Name, Data>: 'a,
{
    type Item = &'a Tree<Name, Data>;

    fn execute(&mut self, _: &mut (), tree: &'a Tree<Name, Data>) -> Self::Item {
        tree
    }
}

/// The [`PostYield`] type of `Tree::iter_tree`.
pub struct IterTreePostYield;
impl PostYield<()> for IterTreePostYield {
    fn execute(&mut self, _: &mut ()) {}
}

type IterTreeResult<'a, Name, Data> =
    TraverseIter<'a, Name, Data, (), IterTreeYield, IterTreePostYield>;

impl<Name, Data: Size> Tree<Name, Data> {
    /// Recursively traverse the tree.
    pub fn iter_node(&self) -> IterTreeResult<'_, Name, Data> {
        self.traverse(IterTreeYield, IterTreePostYield)
    }
}
