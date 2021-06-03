use super::DataTree;
use crate::size::Size;
use rayon::prelude::*;
use std::{
    collections::VecDeque,
    ffi::OsStr,
    fmt::{Debug, Display, Error, Formatter},
    iter::once,
    path::PathBuf,
};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Intermediate format used for construction and inspection of [`DataTree`]'s internal content.
///
/// Unlike `Tree` where the fields are all private, the fields of `TreeReflection`
/// are all public to allow construction in tests.
///
/// **Conversion between `DataTree` and `Reflection`:**
/// * Any `DataTree` can be safely [transmuted](std::mem::transmute) to a valid `Reflection`.
/// * Any `Reflection` can be safely transmuted to a potentially invalid `DataTree`.
/// * To safely convert a `DataTree` into a `Reflection` without the `unsafe` keyword, use
///   [`DataTree::into_reflection`] (it would be slower than using `transmute`).
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

impl<Name, Data: Size> From<DataTree<Name, Data>> for Reflection<Name, Data> {
    fn from(source: DataTree<Name, Data>) -> Self {
        let DataTree {
            name,
            data,
            children,
        } = source;
        let children: Vec<_> = children.into_iter().map(Reflection::from).collect();
        Reflection {
            name,
            data,
            children,
        }
    }
}

impl<Name, Data: Size> DataTree<Name, Data> {
    /// Create reflection.
    pub fn into_reflection(self) -> Reflection<Name, Data> {
        self.into()
    }
}

/// Error that occurs when an attempt to convert a [`Reflection`] into a [`DataTree`] fails.
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

impl<Name, Data> Reflection<Name, Data>
where
    Name: Send,
    Data: Size + Send,
{
    /// Attempting to convert a [`Reflection`] into a valid [`DataTree`].
    pub fn par_try_into_tree(self) -> Result<DataTree<Name, Data>, ConversionError<Name, Data>> {
        let Reflection {
            name,
            data,
            children,
        } = self;
        let children_sum = children.iter().map(|child| child.data).sum();
        if data < children_sum {
            return Err(ConversionError::ExcessiveChildren {
                path: once(name).collect(),
                data,
                children,
                children_sum,
            });
        }
        let children: Result<Vec<_>, _> = children
            .into_par_iter()
            .map(Self::par_try_into_tree)
            .collect();
        let children = match children {
            Ok(children) => children,
            Err(ConversionError::ExcessiveChildren {
                mut path,
                data,
                children,
                children_sum,
            }) => {
                path.push_front(name);
                return Err(ConversionError::ExcessiveChildren {
                    path,
                    data,
                    children,
                    children_sum,
                });
            }
        };
        Ok(DataTree {
            name,
            data,
            children,
        })
    }
}

impl<Name, Data> Reflection<Name, Data>
where
    Name: Send,
    Data: Size + Send,
{
    /// Attempt to transform names and data.
    pub fn par_try_map<TargetName, TargetData, Error, Transform>(
        self,
        transform: Transform,
    ) -> Result<Reflection<TargetName, TargetData>, Error>
    where
        TargetName: Send,
        TargetData: Size + Send + Sync,
        Error: Send,
        Transform: Fn(Name, Data) -> Result<(TargetName, TargetData), Error> + Copy + Sync,
    {
        let Reflection {
            name,
            data,
            children,
        } = self;
        let children = children
            .into_par_iter()
            .map(|child| child.par_try_map(transform))
            .collect::<Result<Vec<_>, _>>()?;
        let (name, data) = transform(name, data)?;
        Ok(Reflection {
            name,
            data,
            children,
        })
    }

    /// Attempt to convert all names from `OsString` to `String`.
    pub fn par_convert_names_to_utf8(self) -> Result<Reflection<String, Data>, Name>
    where
        Name: AsRef<OsStr>,
        Data: Sync,
    {
        self.par_try_map(|name, data| {
            name.as_ref()
                .to_str()
                .map(|name| (name.to_string(), data))
                .ok_or(name)
        })
    }
}
