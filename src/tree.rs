use super::size::Size;

/// Disk usage data of a filesystem tree
#[derive(Debug)]
pub struct Tree<Id, Data: Size> {
    /// Identification of the tree
    pub id: Id,
    /// Disk usage of a file or total disk usage of a folder
    pub data: Data,
    /// Data of children filesystem subtrees
    pub children: Vec<Self>,
}

impl<Id, Data: Size> Tree<Id, Data> {
    /// Extract total disk usage
    fn data(&self) -> Data {
        self.data
    }

    /// Create a tree from a collection of children
    ///
    /// Total disk usage of the subtrees will be assigned to `data`,
    /// this `data` does not include the size of the folder itself,
    /// use [`Self::add_dir_size`] to include it
    pub fn from_children(id: Id, children: Vec<Self>) -> Self {
        let data = children.iter().map(Tree::data).sum();
        Tree { id, data, children }
    }

    /// Add missing directory size
    ///
    /// This function is to be called after [`Self::from_children`]
    pub fn add_dir_size(mut self, dir_size: Data) -> Self {
        self.data += dir_size;
        self
    }
}
