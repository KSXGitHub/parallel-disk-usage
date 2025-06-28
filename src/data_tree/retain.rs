use super::DataTree;
use crate::size;
use rayon::prelude::*;

impl<Name, Size> DataTree<Name, Size>
where
    Self: Send,
    Size: size::Size,
{
    /// Internal function to be used by [`Self::par_retain`].
    fn par_retain_with_depth(
        &mut self,
        current_depth: u64,
        predicate: impl Fn(&Self, u64) -> bool + Copy + Sync,
    ) {
        self.children
            .retain(|child| predicate(child, current_depth));
        let next_depth = current_depth + 1;
        self.children
            .par_iter_mut()
            .for_each(|child| child.par_retain_with_depth(next_depth, predicate))
    }

    /// Recursively cull all descendants that do not satisfy given `predicate`, in parallel.
    pub fn par_retain(&mut self, predicate: impl Fn(&Self, u64) -> bool + Copy + Sync) {
        self.par_retain_with_depth(0, predicate)
    }

    /// Process the tree via [`par_retain`](Self::par_retain) method.
    pub fn into_par_retained(
        mut self,
        predicate: impl Fn(&Self, u64) -> bool + Copy + Sync,
    ) -> Self {
        self.par_retain(predicate);
        self
    }

    /// Recursively cull all descendants whose sizes are too small relative to root.
    #[cfg(feature = "cli")]
    pub fn par_cull_insignificant_data(&mut self, min_ratio: f32)
    where
        Size: Into<u64>,
    {
        let minimal = self.size().into() as f32 * min_ratio;
        self.par_retain(|descendant, _| descendant.size().into() as f32 >= minimal);
    }

    /// Process the tree via [`par_cull_insignificant_data`](Self::par_cull_insignificant_data) method.
    #[cfg(test)]
    #[cfg(feature = "cli")]
    fn into_insignificant_data_par_culled(mut self, min_ratio: f32) -> Self
    where
        Size: Into<u64>,
    {
        self.par_cull_insignificant_data(min_ratio);
        self
    }
}

#[cfg(test)]
#[cfg(feature = "cli")]
mod test;
