use pipe_trait::Pipe;
use std::{iter::FusedIterator, path::PathBuf, slice};

/// List of different hardlinks to the same file.
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

    /// Iterate over the paths inside the list.
    pub fn iter(&self) -> Iter {
        self.0.iter().pipe(Iter)
    }
}

/// [Iterator] over the paths inside a [`LinkPathList`].
#[derive(Debug, Clone)]
pub struct Iter<'a>(slice::Iter<'a, PathBuf>);

impl<'a> Iterator for Iter<'a> {
    type Item = &'a PathBuf;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n)
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last()
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n)
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for Iter<'_> {}
