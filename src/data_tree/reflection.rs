use crate::size;
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
/// Unlike `DataTree` where the fields are all private, the fields of `Reflection`
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
/// **Serialization and deserialization:** _(feature: `json`)_ `Reflection` implements
/// `Serialize` and `Deserialize` traits, this allows functions in `serde_json` to convert
/// a `Reflection` into/from JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "json", serde(rename_all = "kebab-case"))]
pub struct Reflection<Name, Size: size::Size> {
    /// Name of the tree.
    pub name: Name,
    /// Disk usage of a file or total disk usage of a folder.
    pub size: Size,
    /// Data of children filesystem subtrees.
    pub children: Vec<Self>,
}

/// Error that occurs when an attempt to convert a [`Reflection`] into a
/// [`DataTree`](crate::data_tree::DataTree) fails.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConversionError<Name, Size: size::Size> {
    /// When a node's size is less than the sum of its children.
    ExcessiveChildren {
        /// Path from root to the node.
        path: VecDeque<Name>,
        /// Size hold by the node.
        size: Size,
        /// Children of the node.
        children: Vec<Reflection<Name, Size>>,
        /// Sum of size hold by children of the node.
        children_sum: Size,
    },
}

impl<Name, Size> Display for ConversionError<Name, Size>
where
    Name: AsRef<OsStr> + Debug,
    Size: size::Size,
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        use ConversionError::*;
        match self {
            ExcessiveChildren {
                path,
                size,
                children_sum,
                ..
            } => {
                let path = path
                    .iter()
                    .map(PathBuf::from)
                    .fold(PathBuf::new(), |acc, x| acc.join(x));
                write!(
                    formatter,
                    "ExcessiveChildren: {path:?}: {size:?} is less than {children_sum:?}",
                )
            }
        }
    }
}

mod convert;
mod par_methods;
