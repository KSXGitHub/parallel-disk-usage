use super::DataTree;
use crate::size::Size;

/// Reflection of [`DataTree`] used for testing purposes.
///
/// Unlike `Tree` where the fields are all private, the fields of `TreeReflection`
/// are all public to allow construction in tests.
#[derive(Debug, PartialEq, Eq)]
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
