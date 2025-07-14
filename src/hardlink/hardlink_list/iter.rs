use super::HardlinkList;
use crate::{hardlink::LinkPathList, inode::InodeNumber};
use dashmap::{iter::Iter as DashIter, mapref::multiple::RefMulti};
use pipe_trait::Pipe;

/// Iterator over entries in [`HardlinkList`].
#[derive(derive_more::Debug)]
#[debug(bound())]
#[debug("Iter(..)")]
pub struct Iter<'a, Size>(DashIter<'a, InodeNumber, (Size, LinkPathList)>);

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
pub struct Item<'a, Size>(RefMulti<'a, InodeNumber, (Size, LinkPathList)>);

impl<'a, Size> Iterator for Iter<'a, Size> {
    type Item = Item<'a, Size>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Item)
    }
}

impl<'a, Size> Item<'a, Size> {
    /// Number of the inode.
    pub fn ino(&self) -> InodeNumber {
        *self.0.key()
    }

    /// Size of the inode.
    pub fn size(&self) -> &Size {
        &self.0.value().0
    }

    /// Links of the inode.
    pub fn links(&self) -> &LinkPathList {
        &self.0.value().1
    }
}
