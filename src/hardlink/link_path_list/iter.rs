use super::LinkPathList;
use pipe_trait::Pipe;
use std::{iter::FusedIterator, path::PathBuf, slice};

/// [Iterator] over the paths inside a [`LinkPathList`].
#[derive(Debug, Clone)]
pub struct Iter<'a>(slice::Iter<'a, PathBuf>);

impl LinkPathList {
    /// Iterate over the paths inside the list.
    pub fn iter(&self) -> Iter {
        self.0.iter().pipe(Iter)
    }
}

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
