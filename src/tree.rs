use super::size::Size;

/// Disk usage data of a filesystem tree.
#[derive(Debug, PartialEq, Eq)]
pub struct Tree<Name, Data: Size> {
    /// Name of the tree.
    pub name: Name,
    /// Disk usage of a file or total disk usage of a folder.
    pub data: Data,
    /// Data of children filesystem subtrees.
    pub children: Vec<Self>,
}

impl<Name, Data: Size> Tree<Name, Data> {
    /// Extract name
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Extract total disk usage
    pub fn data(&self) -> Data {
        self.data
    }

    /// Extract children
    pub fn children(&self) -> &Vec<Self> {
        &self.children
    }

    /// Create a tree representation of a directory.
    pub fn dir(name: Name, inode_size: Data, children: Vec<Self>) -> Self {
        let data = inode_size + children.iter().map(Tree::data).sum();
        Tree {
            name,
            data,
            children,
        }
    }

    /// Create a tree representation of a file.
    pub fn file(name: Name, data: Data) -> Self {
        Tree {
            name,
            data,
            children: Vec::with_capacity(0),
        }
    }
}
