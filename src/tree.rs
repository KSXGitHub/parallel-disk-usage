use super::size::Size;

/// Disk usage data of a filesystem tree
#[derive(Debug)]
pub struct Tree<Data: Size> {
    /// Disk usage of a file or total disk usage of a folder
    pub data: Data,
    /// Data of children filesystem subtrees
    pub children: Vec<Self>,
}

impl<Data: Size> Tree<Data> {
    /// Extract total disk usage
    fn data(&self) -> Data {
        self.data
    }

    /// Create a tree from a collection of children
    ///
    /// Total disk usage of the subtrees will be assigned to `data`,
    /// this `data` does not include the size of the folder itself,
    /// use [`Self::add_dir_size`] to include it
    pub fn from_children(children: Vec<Tree<Data>>) -> Self {
        let data = children.iter().map(Tree::data).sum();
        Tree { data, children }
    }

    /// Add missing directory size
    ///
    /// This function is to be called after [`Self::from_children`]
    pub fn add_dir_size(mut self, dir_size: Data) -> Self {
        self.data += dir_size;
        self
    }
}
