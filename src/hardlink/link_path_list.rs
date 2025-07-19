mod iter;
mod reflection;

pub use iter::Iter;
pub use reflection::Reflection;

pub use Reflection as LinkPathListReflection;

use std::path::PathBuf;

/// List of different hardlinks to the same file.
///
/// **Reflection:** `LinkPathList` does not implement `PartialEq`, `Eq`, `Hash`,
/// `Deserialize`, and `Serialize` directly. Instead, it can be converted into a
/// [`Reflection`] which implement these traits. Do note that the time complexity
/// of such conversion is O(n) as it has to convert a `Vec` into a `HashSet`.
#[derive(Debug, Clone)]
pub struct LinkPathList(Vec<PathBuf>);

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
