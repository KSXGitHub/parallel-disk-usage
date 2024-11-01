use super::Reflection;
use crate::{data_tree::DataTree, size};

impl<Name, Size: size::Size> From<DataTree<Name, Size>> for Reflection<Name, Size> {
    fn from(source: DataTree<Name, Size>) -> Self {
        let DataTree {
            name,
            size,
            children,
        } = source;
        let children: Vec<_> = children.into_iter().map(Reflection::from).collect();
        Reflection {
            name,
            size,
            children,
        }
    }
}

impl<Name, Size: size::Size> DataTree<Name, Size> {
    /// Create reflection.
    pub fn into_reflection(self) -> Reflection<Name, Size> {
        self.into()
    }
}
