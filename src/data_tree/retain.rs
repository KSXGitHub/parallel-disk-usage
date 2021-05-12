use super::DataTree;
use crate::size::Size;
use rayon::prelude::*;

impl<Name, Data> DataTree<Name, Data>
where
    Self: Send,
    Data: Size,
{
    /// Recursively cull all descendants that do not satisfy given `predicate`, in parallel.
    pub fn par_retain(&mut self, predicate: impl Fn(&Self) -> bool + Copy + Sync) {
        self.children.retain(predicate);
        self.children
            .par_iter_mut()
            .for_each(|child| child.par_retain(predicate));
    }

    /// Process the tree via [`par_retain`](Self::par_retain) method.
    pub fn into_par_retained(mut self, predicate: impl Fn(&Self) -> bool + Copy + Sync) -> Self {
        self.par_retain(predicate);
        self
    }
}
