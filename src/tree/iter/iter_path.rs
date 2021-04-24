use super::{PostYield, TraverseIter, Yield};
use crate::{size::Size, tree::Tree};
use derive_more::{AsMut, AsRef, Deref, From, Into};
use std::{collections::LinkedList, iter::FromIterator, marker::PhantomData};

/// The [Item](Iterator::Item) type of `Tree::iter_path`.
#[derive(Debug, Clone, PartialEq, Eq, AsMut, AsRef, Deref, From, Into)]
pub struct IterPathItem<'a, Name, Data, Path>
where
    Data: Size,
    Path: FromIterator<&'a Name>,
{
    /// Names of the tree's ancestors.
    #[as_mut(ignore)]
    #[as_ref(ignore)]
    pub path: Path,
    /// The current tree.
    #[deref]
    pub tree: &'a Tree<Name, Data>,
}

/// The [`Yield`] type of `Tree::iter_path`.
pub struct IterPathYield<Path>(PhantomData<Path>);
impl<'a, Name, Data, Path> Yield<'a, Name, Data, LinkedList<&'a Name>> for IterPathYield<Path>
where
    Data: Size + 'a,
    Path: FromIterator<&'a Name>,
{
    type Item = IterPathItem<'a, Name, Data, Path>;

    fn execute(
        &mut self,
        parent_path: &mut LinkedList<&'a Name>,
        tree: &'a Tree<Name, Data>,
    ) -> Self::Item {
        let path = parent_path.iter().copied().collect();
        parent_path.push_back(&tree.name);
        IterPathItem { path, tree }
    }
}

/// The [`PostYield`] type of `Tree::iter_path`.
pub struct IterPathPostYield;
impl<'a, Name> PostYield<LinkedList<&'a Name>> for IterPathPostYield {
    fn execute(&mut self, parent_path: &mut LinkedList<&'a Name>) {
        parent_path.pop_back();
    }
}

/// The return type of `Tree::iter_path`.
pub type IterPathResult<'a, Name, Data, Path> =
    TraverseIter<'a, Name, Data, LinkedList<&'a Name>, IterPathYield<Path>, IterPathPostYield>;

impl<Name, Data: Size> Tree<Name, Data> {
    /// Recursively traverse the tree with parent path.
    pub fn iter_path<'a, Path>(&'a self) -> IterPathResult<'a, Name, Data, Path>
    where
        Path: FromIterator<&'a Name>,
    {
        self.traverse(IterPathYield(PhantomData), IterPathPostYield)
    }
}
