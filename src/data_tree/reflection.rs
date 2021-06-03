use crate::size::Size;
use std::{
    collections::VecDeque,
    ffi::OsStr,
    fmt::{Debug, Display, Error, Formatter},
    path::PathBuf,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Intermediate format used for construction and inspection of
/// [`DataTree`](crate::data_tree::DataTree)'s internal content.
///
/// Unlike `Tree` where the fields are all private, the fields of `TreeReflection`
/// are all public to allow construction in tests.
///
/// **Conversion between `DataTree` and `Reflection`:**
/// * Any `DataTree` can be safely [transmuted](std::mem::transmute) to a valid `Reflection`.
/// * Any `Reflection` can be safely transmuted to a potentially invalid `DataTree`.
/// * To safely convert a `DataTree` into a `Reflection` without the `unsafe` keyword, use
///   [`DataTree::into_reflection`](crate::data_tree::DataTree::into_reflection)
///   (it would be slower than using `transmute`).
/// * To safely convert a `Reflection` into a valid `DataTree`,
///   use [`par_try_into_tree`](Self::par_try_into_tree).
///
/// **Serialization and deserialization:** Requires enabling the `json` feature to enable `serde`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct Reflection<Name, Data: Size> {
    /// Name of the tree.
    pub name: Name,
    /// Disk usage of a file or total disk usage of a folder.
    pub data: Data,
    /// Data of children filesystem subtrees.
    pub children: Vec<Self>,
}

/// Error that occurs when an attempt to convert a [`Reflection`] into a
/// [`DataTree`](crate::data_tree::DataTree) fails.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConversionError<Name, Data: Size> {
    /// When a node's data is less than the sum of its children.
    ExcessiveChildren {
        /// Path from root to the node.
        path: VecDeque<Name>,
        /// Data hold by the node.
        data: Data,
        /// Children of the node.
        children: Vec<Reflection<Name, Data>>,
        /// Sum of data hold by children of the node.
        children_sum: Data,
    },
}

impl<Name, Data> Display for ConversionError<Name, Data>
where
    Name: AsRef<OsStr> + Debug,
    Data: Size,
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        use ConversionError::*;
        match self {
            ExcessiveChildren {
                path,
                data,
                children_sum,
                ..
            } => {
                let path = path
                    .iter()
                    .map(PathBuf::from)
                    .fold(PathBuf::new(), |acc, x| acc.join(x));
                write!(
                    formatter,
                    "ExcessiveChildren: {path:?}: {data:?} is less than {sum:?}",
                    path = path,
                    data = data,
                    sum = children_sum,
                )
            }
        }
    }
}

mod convert;
mod par_methods;
