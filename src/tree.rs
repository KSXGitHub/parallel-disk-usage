pub mod iter;

pub use iter::{Item as IterItem, Iter};

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
    /// Extract total disk usage
    fn data(&self) -> Data {
        self.data
    }

    /// Create a tree from a collection of children.
    ///
    /// Total disk usage of the subtrees will be assigned to `data`,
    /// this `data` does not include the size of the folder itself,
    /// use [`Self::add_dir_size`] to include it.
    pub fn from_children(name: Name, children: Vec<Self>) -> Self {
        let data = children.iter().map(Tree::data).sum();
        Tree {
            name,
            data,
            children,
        }
    }

    /// Add missing directory size.
    ///
    /// This function is to be called after [`Self::from_children`].
    pub fn add_dir_size(mut self, dir_size: Data) -> Self {
        self.data += dir_size;
        self
    }
}
