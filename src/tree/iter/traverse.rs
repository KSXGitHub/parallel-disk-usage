use crate::{size::Size, tree::Tree};
use std::{collections::LinkedList, mem::replace, slice};

/// Pre-process the record and produce item to be emitted by [the iterator](TraverseIter).
///
/// This trait is executed when the iterator finds a tree and descends.
pub trait Yield<'a, Name, Data: Size, Record> {
    /// Iterator item.
    type Item;

    /// Pre-process the record and produce item to be emitted by the iterator.
    fn execute(&mut self, record: &mut Record, tree: &'a Tree<Name, Data>) -> Self::Item;
}

/// Post-process the record.
///
/// This trait is executed when the iterator ascends one level.
pub trait PostYield<Record> {
    fn execute(&mut self, record: &mut Record);
}

type SubIter<'a, Name, Data> = slice::Iter<'a, Tree<Name, Data>>;

/// [`Iterator`] type of [`Tree`]. Created by calling `Tree::iter`.
#[derive(Debug, Clone)]
pub struct TraverseIter<'a, Name, Data, Record, OnYield, OnPostYield>
where
    Data: Size,
    Record: Default,
    OnYield: Yield<'a, Name, Data, Record>,
    OnPostYield: PostYield<Record>,
{
    waiting_tree: Option<&'a Tree<Name, Data>>,
    children_iter: SubIter<'a, Name, Data>,
    stacked_children_iter: LinkedList<SubIter<'a, Name, Data>>,
    record: Record,
    on_yield: OnYield,
    on_post_yield: OnPostYield,
}

impl<'a, Name, Data, Record, OnYield, OnPostYield> Iterator
    for TraverseIter<'a, Name, Data, Record, OnYield, OnPostYield>
where
    Data: Size,
    Record: Default,
    OnYield: Yield<'a, Name, Data, Record>,
    OnPostYield: PostYield<Record>,
{
    type Item = OnYield::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(tree) = self.waiting_tree {
            self.waiting_tree = None;
            Some(self.on_yield.execute(&mut self.record, tree))
        } else if let Some(tree) = self.children_iter.next() {
            self.waiting_tree = Some(tree);
            let prev_children_iter = replace(&mut self.children_iter, tree.children.iter());
            self.stacked_children_iter.push_back(prev_children_iter);
            self.next()
        } else if let Some(next_children_iter) = self.stacked_children_iter.pop_back() {
            self.children_iter = next_children_iter;
            self.on_post_yield.execute(&mut self.record);
            self.next()
        } else {
            None
        }
    }
}

impl<Name, Data: Size> Tree<Name, Data> {
    /// Recursively traverse the tree.
    pub fn traverse<'a, Record, OnYield, OnPostYield>(
        &'a self,
        on_yield: OnYield,
        on_post_yield: OnPostYield,
    ) -> TraverseIter<'a, Name, Data, Record, OnYield, OnPostYield>
    where
        Record: Default,
        OnYield: Yield<'a, Name, Data, Record>,
        OnPostYield: PostYield<Record>,
    {
        TraverseIter {
            waiting_tree: Some(self),
            children_iter: self.children.iter(),
            stacked_children_iter: LinkedList::new(),
            record: Default::default(),
            on_yield,
            on_post_yield,
        }
    }
}
