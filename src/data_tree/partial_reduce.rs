use super::DataTree;
use crate::size::Size;
use assert_cmp::debug_assert_op_expr;
use rayon::prelude::*;

/// Parameter to pass to `name_reduced` callback of [`par_partial_reduce`](DataTree::par_partial_reduce).
#[derive(Debug, Clone, Copy)]
pub struct NameReducedParam<'tree, 'local, Name, Data: Size> {
    /// Children chosen by `predicate`.
    pub reduced_children: &'local Vec<DataTree<Name, Data>>,
    /// Total of data of chosen children.
    pub reduced_data_sum: Data,
    /// Data of the parent.
    pub parent_data: Data,
    /// Name of the parent.
    pub parent_name: &'tree Name,
}

/// Parameter to pass to `predicate` callback of [`par_partial_reduce`](DataTree::par_partial_reduce).
#[derive(Debug, Clone, Copy)]
pub struct PredicateParam<'a, Name, Data: Size> {
    /// The current child.
    pub child: &'a DataTree<Name, Data>,
    /// Data of the parent.
    pub parent_data: Data,
    /// Name of the parent.
    pub parent_name: &'a Name,
}

impl<Name, Data> DataTree<Name, Data>
where
    Self: Send,
    Name: Send + Sync,
    Data: Size + Send + Sync,
{
    /// Reduce some children into one, recursively.
    pub fn par_partial_reduce(
        self,
        name_reduced: impl Fn(NameReducedParam<Name, Data>) -> Name + Copy + Send + Sync,
        predicate: impl Fn(PredicateParam<Name, Data>) -> bool + Copy + Send + Sync,
    ) -> Self {
        if self.children().len() < 3 {
            return self;
        }
        let DataTree {
            name,
            data,
            children,
        } = self;
        let (reduced, unreduced): (Vec<_>, Vec<_>) = children.into_par_iter().partition(|child| {
            predicate(PredicateParam {
                child,
                parent_data: data,
                parent_name: &name,
            })
        });
        let mut children: Vec<_> = unreduced
            .into_par_iter()
            .map(|child| child.par_partial_reduce(name_reduced, predicate))
            .collect();
        let reduced_data = reduced.iter().map(|child| child.data()).sum();
        let reduced_name = name_reduced(NameReducedParam {
            reduced_children: &reduced,
            reduced_data_sum: reduced_data,
            parent_data: data,
            parent_name: &name,
        });
        children.push(DataTree {
            name: reduced_name,
            data: reduced_data,
            children: Vec::with_capacity(0),
        });
        if cfg!(debug_assertions) {
            debug_assert_op_expr!(
                children.iter().map(DataTree::data).sum::<Data>(),
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

    /// Recursively reduce children whose data are under certain ratio.
    pub(crate) fn par_partial_reduce_insignificant_data(
        self,
        minimal_ratio: f32,
        name_reduced: impl Fn(NameReducedParam<Name, Data>) -> Name + Copy + Send + Sync,
    ) -> Self
    where
        Data: Into<u64>,
    {
        self.par_partial_reduce(name_reduced, |param| {
            let minimal = param.parent_data.into() as f32 * minimal_ratio;
            let actual = param.child.data().into() as f32;
            minimal >= actual
        })
    }
}
