use super::{HardlinkList, Value};
use crate::{hardlink::LinkPathList, inode::InodeNumber};
use dashmap::{iter::Iter as DashIter, mapref::multiple::RefMulti};
use pipe_trait::Pipe;

/// Iterator over entries in [`HardlinkList`].
#[derive(derive_more::Debug)]
#[debug(bound())]
#[debug("Iter(..)")]
pub struct Iter<'a, Size>(DashIter<'a, InodeNumber, Value<Size>>);

impl<Size> HardlinkList<Size> {
    /// Iterate over the recorded entries.
    pub fn iter(&self) -> Iter<Size> {
        self.0.iter().pipe(Iter)
    }
}

/// [Item](Iterator::Item) of [`Iter`].
#[derive(derive_more::Debug)]
#[debug(bound())]
#[debug("Item(..)")]
pub struct Item<'a, Size>(RefMulti<'a, InodeNumber, Value<Size>>);

impl<'a, Size> Iterator for Iter<'a, Size> {
    type Item = Item<'a, Size>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Item)
    }
}

impl<'a, Size> Item<'a, Size> {
    /// The inode number of the file.
    pub fn ino(&self) -> InodeNumber {
        *self.0.key()
    }

    /// Size of the file.
    pub fn size(&self) -> &Size {
        &self.0.value().size
    }

    /// Total number of links of the file, both listed (in [`Self::paths`]) and unlisted.
    pub fn links(&self) -> u64 {
        self.0.value().links
    }

    /// Paths to the detected links of the file.
    pub fn paths(&self) -> &LinkPathList {
        &self.0.value().paths
    }
}
