use super::{ConversionError, Reflection};
use crate::{data_tree::DataTree, size};
use rayon::prelude::*;
use std::{ffi::OsStr, iter::once};

impl<Name, Size> Reflection<Name, Size>
where
    Name: Send,
    Size: size::Size + Send,
{
    /// Attempting to convert a [`Reflection`] into a valid [`DataTree`].
    pub fn par_try_into_tree(self) -> Result<DataTree<Name, Size>, ConversionError<Name, Size>> {
        let Reflection {
            name,
            size,
            children,
        } = self;
        let excess_child = children
            .iter()
            .enumerate()
            .find(|(_, child)| child.size > size);
        if let Some((index, _)) = excess_child {
            let path = once(name).collect();
            let mut children = children;
            let child = children.swap_remove(index); // this still does the unnecessary work of swapping the elements, how to skip it?
            return Err(ConversionError::ExcessiveChildren { path, size, child });
        }
        let children: Result<Vec<_>, _> = children
            .into_par_iter()
            .map(Self::par_try_into_tree)
            .collect();
        let children = match children {
            Ok(children) => children,
            Err(ConversionError::ExcessiveChildren {
                mut path,
                size,
                child,
            }) => {
                path.push_front(name);
                return Err(ConversionError::ExcessiveChildren { path, size, child });
            }
        };
        Ok(DataTree {
            name,
            size,
            children,
        })
    }

    /// Attempt to transform names and sizes.
    pub fn par_try_map<TargetName, TargetSize, Error, Transform>(
        self,
        transform: Transform,
    ) -> Result<Reflection<TargetName, TargetSize>, Error>
    where
        TargetName: Send,
        TargetSize: size::Size + Send + Sync,
        Error: Send,
        Transform: Fn(Name, Size) -> Result<(TargetName, TargetSize), Error> + Copy + Sync,
    {
        let Reflection {
            name,
            size,
            children,
        } = self;
        let children = children
            .into_par_iter()
            .map(|child| child.par_try_map(transform))
            .collect::<Result<Vec<_>, _>>()?;
        let (name, size) = transform(name, size)?;
        Ok(Reflection {
            name,
            size,
            children,
        })
    }

    /// Attempt to convert all names from `OsString` to `String`.
    pub fn par_convert_names_to_utf8(self) -> Result<Reflection<String, Size>, Name>
    where
        Name: AsRef<OsStr>,
        Size: Sync,
    {
        self.par_try_map(|name, size| {
            name.as_ref()
                .to_str()
                .map(|name| (name.to_string(), size))
                .ok_or(name)
        })
    }
}
