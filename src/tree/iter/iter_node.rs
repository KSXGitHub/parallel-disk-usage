use super::{PostYield, TraverseIter, Yield};
use crate::{size::Size, tree::Tree};

/// The [`Yield`] type of `Tree::iter_node`.
pub struct IterNodeYield;
impl<'a, Name, Data> Yield<'a, Name, Data, ()> for IterNodeYield
where
    Data: Size,
    Tree<Name, Data>: 'a,
{
    type Item = &'a Tree<Name, Data>;

    fn execute(&mut self, _: &mut (), node: &'a Tree<Name, Data>) -> Self::Item {
        node
    }
}

/// The [`PostYield`] type of `Tree::iter_node`.
pub struct IterNodePostYield;
impl PostYield<()> for IterNodePostYield {
    fn execute(&mut self, _: &mut ()) {}
}

type IterNodeResult<'a, Name, Data> =
    TraverseIter<'a, Name, Data, (), IterNodeYield, IterNodePostYield>;

impl<Name, Data: Size> Tree<Name, Data> {
    /// Recursively traverse the tree.
    pub fn iter_node(&self) -> IterNodeResult<'_, Name, Data> {
        self.traverse(IterNodeYield, IterNodePostYield)
    }
}
