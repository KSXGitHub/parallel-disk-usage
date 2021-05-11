use super::DataTree;
use crate::size::Size;
use assert_cmp::debug_assert_op_expr;
use rayon::prelude::*;

impl<Name, Data: Size> DataTree<Name, Data> {
    /// Reduce some children into one, recursively.
    pub fn par_partial_reduce(
        self,
        name_reduced: impl Fn(&Vec<Self>, Data) -> Name + Copy + Send + Sync,
        predicate: impl Fn(&Self) -> bool + Copy + Send + Sync,
    ) -> Self
    where
        Self: Send,
        Data: Send,
    {
        if self.children().len() < 3 {
            return self;
        }
        let DataTree {
            name,
            data,
            children,
        } = self;
        let (reduced, unreduced): (Vec<_>, Vec<_>) = children.into_par_iter().partition(predicate);
        let mut children: Vec<_> = unreduced
            .into_par_iter()
            .map(|child| child.par_partial_reduce(name_reduced, predicate))
            .collect();
        let reduced_data = reduced.iter().map(|child| child.data()).sum();
        let reduced_name = name_reduced(&reduced, reduced_data);
        children.push(DataTree {
            name: reduced_name,
            data: reduced_data,
            children: Vec::with_capacity(0),
        });
        if cfg!(debug_assertions) {
            debug_assert_op_expr!(
                children.iter().map(|child| child.data()).sum::<Data>(),
                ==,
                data
            );
        }
        DataTree {
            name,
            data,
            children,
        }
    }
}
