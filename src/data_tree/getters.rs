use super::DataTree;
use crate::size;

impl<Name, Size: size::Size> DataTree<Name, Size> {
    /// Extract name
    #[inline]
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Get mutable reference to name.
    #[inline]
    pub fn name_mut(&mut self) -> &mut Name {
        &mut self.name
    }

    /// Extract total disk usage
    #[inline]
    pub fn size(&self) -> Size {
        self.size
    }

    /// Extract children
    #[inline]
    pub fn children(&self) -> &Vec<Self> {
        &self.children
    }
}
