mod iter;
mod reflection;

pub use iter::Iter;
pub use reflection::Reflection;

pub use Reflection as LinkPathListReflection;

use std::path::PathBuf;

/// List of different hardlinks to the same file.
///
/// **Serialization and deserialization:** _(feature: `json`)_ `LinkPathList` does not implement
/// `Serialize` and `Deserialize` traits directly, instead, it can be converted into/from a
/// [`Reflection`] which implements these traits.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinkPathList(
    Vec<PathBuf>, // TODO: benchmark against LinkedList<PathBuf>
);

impl LinkPathList {
    /// Create a list of a single path.
    pub(crate) fn single(path: PathBuf) -> Self {
        LinkPathList(vec![path])
    }

    /// Add a path to the list.
    pub(crate) fn add(&mut self, path: PathBuf) {
        self.0.push(path)
    }

    /// Get the number of paths inside the list.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Create reflection.
    pub fn into_reflection(self) -> Reflection {
        self.into()
    }
}
