use super::{ConversionError, Reflection};
use crate::{data_tree::DataTree, size::Size};
use rayon::prelude::*;
use std::{ffi::OsStr, iter::once};

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
