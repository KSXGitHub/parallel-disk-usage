use super::DataTree;
use crate::size::Size;

impl<Name, Data: Size> DataTree<Name, Data> {
    /// Extract name
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Get mutable reference to name.
    pub fn name_mut(&mut self) -> &mut Name {
        &mut self.name
    }

    /// Extract total disk usage
    pub fn data(&self) -> Data {
        self.data
    }

    /// Extract children
    pub fn children(&self) -> &Vec<Self> {
        &self.children
    }
}
