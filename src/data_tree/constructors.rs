use super::DataTree;
use crate::size;

impl<Name, Size: size::Size> DataTree<Name, Size> {
    /// Create a tree representation of a directory.
    #[inline]
    pub fn dir(name: Name, inode_size: Size, children: Vec<Self>) -> Self {
        let size = inode_size + children.iter().map(DataTree::size).sum();
        DataTree {
            name,
            size,
            children,
        }
    }

    /// Create a tree representation of a file.
    #[inline]
    pub fn file(name: Name, size: Size) -> Self {
        DataTree {
            name,
            size,
            children: Vec::new(),
        }
    }

    /// Create a directory constructor of fixed inode size.
    #[inline]
    pub fn fixed_size_dir_constructor(inode_size: Size) -> impl Fn(Name, Vec<Self>) -> Self
    where
        Size: Copy,
    {
        move |name, children| DataTree::dir(name, inode_size, children)
    }
}
