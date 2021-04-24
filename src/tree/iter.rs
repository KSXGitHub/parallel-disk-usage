use super::Tree;
use crate::size::Size;
use derive_more::{AsMut, AsRef, Deref, From, Into};
use std::{collections::LinkedList, mem::replace, slice};

/// [Item](Iterator::Item) type of [`TreeIter`].
#[derive(Debug, Clone, PartialEq, Eq, AsMut, AsRef, Deref, From, Into)]
pub struct Item<'a, Name, Data: Size> {
    /// Names of the tree's ancestors.
    #[as_mut(ignore)]
    #[as_ref(ignore)]
    pub path: Vec<&'a Name>,
    /// The current tree.
    #[deref]
    pub tree: &'a Tree<Name, Data>,
}

type SubIter<'a, Name, Data> = slice::Iter<'a, Tree<Name, Data>>;

/// [`Iterator`] type of [`Tree`]. Created by calling `Tree::iter`.
#[derive(Debug, Clone)]
pub struct Iter<'a, Name, Data: Size> {
    waiting_tree: Option<&'a Tree<Name, Data>>,
    children_iter: SubIter<'a, Name, Data>,
    stacked_children_iter: LinkedList<SubIter<'a, Name, Data>>,
    path: LinkedList<&'a Name>,
}

impl<'a, Name, Data: Size> Iterator for Iter<'a, Name, Data> {
    type Item = Item<'a, Name, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! path {
            () => {
                self.path.iter().copied().collect::<Vec<_>>()
            };
        }
        if let Some(tree) = self.waiting_tree {
            self.waiting_tree = None;
            let path = path!();
            self.path.push_back(&tree.name);
            Some(Item { path, tree })
        } else if let Some(tree) = self.children_iter.next() {
            self.waiting_tree = Some(tree);
            let prev_children_iter = replace(&mut self.children_iter, tree.children.iter());
            self.stacked_children_iter.push_back(prev_children_iter);
            self.next()
        } else if let Some(next_children_iter) = self.stacked_children_iter.pop_back() {
            self.children_iter = next_children_iter;
            self.path.pop_back();
            self.next()
        } else {
            None
        }
    }
}

impl<Name, Data: Size> Tree<Name, Data> {
    /// Recursively traverse the tree.
    pub fn iter(&self) -> Iter<'_, Name, Data> {
        Iter {
            waiting_tree: Some(self),
            children_iter: self.children.iter(),
            stacked_children_iter: LinkedList::new(),
            path: LinkedList::new(),
        }
    }
}
