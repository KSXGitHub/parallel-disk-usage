use super::{PostYield, TraverseIter, Yield};
use crate::{size::Size, tree::Tree};
use derive_more::{AsMut, AsRef, Deref, From, Into};
use std::{collections::LinkedList, iter::FromIterator, marker::PhantomData};

/// The [`Item`](Iterator::Item) type of `Tree::iter_path_node`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsMut, AsRef, Deref, From, Into)]
pub struct IterPathNodeItem<'a, Name, Data, Path>
where
    Data: Size,
    Path: FromIterator<&'a Name>,
{
    /// Names of the node's ancestors.
    #[as_mut(ignore)]
    #[as_ref(ignore)]
    pub path: Path,
    /// The current node.
    #[deref]
    pub node: &'a Tree<Name, Data>,
}

/// The [`Yield`] type of `Tree::iter_path_node`.
pub struct IterPathNodeYield<Path>(PhantomData<Path>);
impl<'a, Name, Data, Path> Yield<'a, Name, Data, LinkedList<&'a Name>> for IterPathNodeYield<Path>
where
    Data: Size,
    Path: FromIterator<&'a Name>,
    Tree<Name, Data>: 'a,
{
    type Item = IterPathNodeItem<'a, Name, Data, Path>;

    fn execute(
        &mut self,
        parent_path: &mut LinkedList<&'a Name>,
        node: &'a Tree<Name, Data>,
    ) -> Self::Item {
        let path = parent_path.iter().copied().collect();
        parent_path.push_back(&node.name);
        IterPathNodeItem { path, node }
    }
}

/// The [`PostYield`] type of `Tree::iter_path_node`.
pub struct IterPathNodePostYield;
impl<'a, Name> PostYield<LinkedList<&'a Name>> for IterPathNodePostYield {
    fn execute(&mut self, parent_path: &mut LinkedList<&'a Name>) {
        parent_path.pop_back();
    }
}

/// The return type of `Tree::iter_path_node`.
pub type IterPathResult<'a, Name, Data, Path> = TraverseIter<
    'a,
    Name,
    Data,
    LinkedList<&'a Name>,
    IterPathNodeYield<Path>,
    IterPathNodePostYield,
>;

impl<Name, Data: Size> Tree<Name, Data> {
    /// Recursively traverse the tree with parent path.
    pub fn iter_path_node<'a, Path>(&'a self) -> IterPathResult<'a, Name, Data, Path>
    where
        Path: FromIterator<&'a Name>,
    {
        self.traverse(IterPathNodeYield(PhantomData), IterPathNodePostYield)
    }
}
