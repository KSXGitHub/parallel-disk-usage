pub mod reflection;

pub use reflection::Reflection;

use crate::{hardlink::LinkPathList, inode::InodeNumber, size};
use dashmap::{iter::Iter as DashIter, mapref::multiple::RefMulti, DashMap};
use derive_more::{Display, Error};
use pipe_trait::Pipe;
use std::{fmt::Debug, path::Path};

/// Storage to be used by [`crate::hook::RecordHardlink`].
///
/// **Serialization and deserialization:** _(feature: `json`)_ `HardlinkList` does not implement
/// `Serialize` and `Deserialize` traits directly, instead, it can be converted into/from a
/// [`Reflection`] which implements these traits.
#[derive(Debug, Clone)]
pub struct HardlinkList<Size>(
    /// Map an inode number to its size and detected paths.
    DashMap<InodeNumber, (Size, LinkPathList)>, // TODO: benchmark against Mutex<HashMap<InodeNumber, (Size, LinkPathList)>>
);

impl<Size> HardlinkList<Size> {
    /// Create a new record.
    pub fn new() -> Self {
        HardlinkList(DashMap::new())
    }

    /// Iterate over the recorded entries.
    pub fn iter(&self) -> Iter<Size> {
        self.0.iter().pipe(Iter)
    }
}

impl<Size> Default for HardlinkList<Size> {
    fn default() -> Self {
        HardlinkList::new()
    }
}

/// Error that occurs when a different size was detected for the same [`ino`](std::os::unix::fs::MetadataExt::ino).
#[derive(Debug, Display, Error)]
#[display(bound(Size: Debug))]
#[display("Size for inode {ino} changed from {recorded:?} to {detected:?}")]
pub struct SizeConflictError<Size> {
    pub ino: InodeNumber,
    pub recorded: Size,
    pub detected: Size,
}

/// Error that occurs when it fails to add an item to [`HardlinkList`].
#[derive(Debug, Display, Error)]
#[display(bound(Size: Debug))]
#[non_exhaustive]
pub enum AddError<Size> {
    SizeConflict(SizeConflictError<Size>),
}

impl<Size> HardlinkList<Size>
where
    Size: size::Size,
{
    /// Add an entry to the record.
    pub(crate) fn add(
        &self,
        ino: InodeNumber,
        size: Size,
        path: &Path,
    ) -> Result<(), AddError<Size>> {
        let mut size_assertion = Ok(());
        self.0
            .entry(ino)
            .and_modify(|(recorded, paths)| {
                let (detected, recorded) = (size, *recorded);
                if size == recorded {
                    paths.add(path.to_path_buf());
                } else {
                    size_assertion = Err(SizeConflictError {
                        ino,
                        recorded,
                        detected,
                    });
                }
            })
            .or_insert_with(|| (size, path.to_path_buf().pipe(LinkPathList::single)));
        size_assertion.map_err(AddError::SizeConflict)
    }
}

/// Iterator over entries in [`HardlinkList`].
#[derive(derive_more::Debug)]
#[debug(bound())]
#[debug("Iter(..)")]
pub struct Iter<'a, Size>(DashIter<'a, InodeNumber, (Size, LinkPathList)>);

/// [Item](Iterator::Item) of [`Iter`].
#[derive(derive_more::Debug)]
#[debug(bound())]
#[debug("IterItem(..)")]
pub struct IterItem<'a, Size>(RefMulti<'a, InodeNumber, (Size, LinkPathList)>);

impl<'a, Size> Iterator for Iter<'a, Size> {
    type Item = IterItem<'a, Size>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(IterItem)
    }
}

impl<'a, Size> IterItem<'a, Size> {
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
