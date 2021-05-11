use super::DataTree;
use crate::size::Size;

impl<Name, Data: Size> DataTree<Name, Data> {
    /// Create a tree representation of a directory.
    pub fn dir(name: Name, inode_size: Data, children: Vec<Self>) -> Self {
        let data = inode_size + children.iter().map(DataTree::data).sum();
        DataTree {
            name,
            data,
            children,
        }
    }

    /// Create a tree representation of a file.
    pub fn file(name: Name, data: Data) -> Self {
        DataTree {
            name,
            data,
            children: Vec::with_capacity(0),
        }
    }

    /// Create a directory constructor of fixed inode size.
    pub fn fixed_size_dir_constructor(inode_size: Data) -> impl Fn(Name, Vec<Self>) -> Self
    where
        Data: Copy,
    {
        move |name, children| DataTree::dir(name, inode_size, children)
    }
}
