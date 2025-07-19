mod iter;
mod reflection;

pub use iter::Iter;
pub use reflection::Reflection;

pub use Reflection as LinkPathListReflection;

use std::path::PathBuf;

/// List of different hardlinks to the same file.
///
/// **Reflection:** `LinkPathList` does not implement `PartialEq`, `Eq`,
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

    /// Create a list of many paths.
    #[cfg(test)]
    pub(crate) fn many(paths: impl IntoIterator<Item: Into<PathBuf>>) -> Self {
        let paths: Vec<_> = paths.into_iter().map(Into::into).collect();
        assert!(!paths.is_empty(), "paths must not be empty");
        LinkPathList(paths)
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

#[cfg(test)]
mod tests {
    use super::LinkPathList;
    use pipe_trait::Pipe;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn item_order_is_irrelevant_to_equality() {
        let a = ["3", "4", "0", "2", "1"]
            .pipe(LinkPathList::many)
            .into_reflection();
        let b = ["4", "0", "3", "2", "1"]
            .pipe(LinkPathList::many)
            .into_reflection();
        let c = ["0", "1", "2", "3", "4"]
            .pipe(LinkPathList::many)
            .into_reflection();
        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(a, c);
    }

    #[test]
    fn item_absent_cause_inequality() {
        let a = ["0", "1", "2", "3"]
            .pipe(LinkPathList::many)
            .into_reflection();
        let b = ["0", "1", "2", "3", "4"]
            .pipe(LinkPathList::many)
            .into_reflection();
        assert_ne!(a, b);
        assert_ne!(b, a);
    }

    #[test]
    fn item_difference_cause_inequality() {
        let a = ["0", "1", "2", "3", "5"]
            .pipe(LinkPathList::many)
            .into_reflection();
        let b = ["0", "1", "2", "3", "4"]
            .pipe(LinkPathList::many)
            .into_reflection();
        assert_ne!(a, b);
        assert_ne!(b, a);
    }
}
