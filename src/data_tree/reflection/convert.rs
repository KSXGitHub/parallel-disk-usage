use super::Reflection;
use crate::{data_tree::DataTree, size::Size};

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
