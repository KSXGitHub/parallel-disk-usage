use super::DataTree;
use crate::size;
use rayon::prelude::*;
use std::cmp::Ordering;

impl<Name, Size> DataTree<Name, Size>
where
    Self: Send,
    Size: size::Size,
{
    /// Sort all descendants recursively, in parallel.
    pub fn par_sort_by(&mut self, compare: impl Fn(&Self, &Self) -> Ordering + Copy + Sync) {
        self.children
            .par_iter_mut()
            .for_each(|child| child.par_sort_by(compare));
        self.children.sort_by(compare);
    }

    /// Process the tree via [`par_sort_by`](Self::par_sort_by) method.
    #[inline]
    pub fn into_par_sorted(
        mut self,
        compare: impl Fn(&Self, &Self) -> Ordering + Copy + Sync,
    ) -> Self {
        self.par_sort_by(compare);
        self
    }
}
