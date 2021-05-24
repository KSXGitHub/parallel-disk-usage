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

    /// Recursively cull all descendants whose data are too small relative to root.
    pub(crate) fn par_cull_insignificant_data(&mut self, min_ratio: f32)
    where
        Data: Into<u64>,
    {
        let minimal = self.data().into() as f32 * min_ratio;
        self.par_retain(|descendant| descendant.data().into() as f32 >= minimal);
    }

    /// Process the tree via [`par_cull_insignificant_data`](Self::par_cull_insignificant_data) method.
    #[cfg(test)]
    fn into_insignificant_data_par_culled(mut self, min_ratio: f32) -> Self
    where
        Data: Into<u64>,
    {
        self.par_cull_insignificant_data(min_ratio);
        self
    }
}

#[cfg(test)]
mod test;
