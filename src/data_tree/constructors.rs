use super::DataTree;
use crate::size;

impl<Name, Size: size::Size> DataTree<Name, Size> {
    /// Create a tree representation of a directory.
    pub fn dir(name: Name, inode_size: Size, children: Vec<Self>, depth: u64) -> Self {
        let size = inode_size + children.iter().map(DataTree::size).sum();
        let children = if depth > 0 { children } else { Vec::new() };
        DataTree {
            name,
            size,
            children,
        }
    }

    /// Create a tree representation of a file.
    pub fn file(name: Name, size: Size) -> Self {
        DataTree {
            name,
            size,
            children: Vec::new(),
        }
    }

    /// Create a directory constructor of fixed inode size.
    pub fn fixed_size_dir_constructor(inode_size: Size) -> impl Fn(Name, Vec<Self>) -> Self
    where
        Size: Copy,
    {
        move |name, children| DataTree::dir(name, inode_size, children, 1)
    }
}
