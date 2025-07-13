use crate::size;
use dashmap::{iter::Iter as DashIter, mapref::multiple::RefMulti, DashMap};
use derive_more::{Display, Error};
use pipe_trait::Pipe;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

/// Storage to be used by [`crate::hook::RecordHardLink`].
#[derive(Debug, Clone)]
pub struct RecordHardLinkStorage<Size>(
    /// Map an inode number to its size and detected paths.
    DashMap<u64, (Size, Vec<PathBuf>)>, // TODO: benchmark against Mutex<HashMap<u64, (Size, Vec<PathBuf>)>>
);

impl<Size> RecordHardLinkStorage<Size> {
    /// Create a new record.
    pub fn new() -> Self {
        RecordHardLinkStorage(DashMap::new())
    }

    /// Iterate over the recorded entries.
    pub fn iter(&self) -> Iter<Size> {
        self.0.iter().pipe(Iter)
    }
}

impl<Size> Default for RecordHardLinkStorage<Size> {
    fn default() -> Self {
        RecordHardLinkStorage::new()
    }
}

/// Error that occurs when a different size was detected for the same [`ino`](std::os::unix::fs::MetadataExt::ino).
#[derive(Debug, Display, Error)]
#[display(bound(Size: Debug))]
#[display("Size for inode {ino} changed from {recorded:?} to {detected:?}")]
pub struct SizeConflictError<Size> {
    pub ino: u64, // TODO: define a newtype of ino.
    pub recorded: Size,
    pub detected: Size,
}

/// Error type of [`RecordHardLinkStorage::add`].
#[derive(Debug, Display, Error)]
#[display(bound(Size: Debug))]
#[non_exhaustive]
pub enum AddError<Size> {
    SizeConflict(SizeConflictError<Size>),
}

impl<Size> RecordHardLinkStorage<Size>
where
    Size: size::Size,
{
    /// Add an entry to the record.
    pub(crate) fn add(&self, ino: u64, size: Size, path: &Path) -> Result<(), AddError<Size>> {
        let mut size_assertion = Ok(());
        self.0
            .entry(ino)
            .and_modify(|(recorded, paths)| {
                let (detected, recorded) = (size, *recorded);
                if size == recorded {
                    paths.push(path.to_path_buf());
                } else {
                    size_assertion = Err(SizeConflictError {
                        ino,
                        recorded,
                        detected,
                    });
                }
            })
            .or_insert_with(|| (size, vec![path.to_path_buf()]));
        size_assertion.map_err(AddError::SizeConflict)
    }
}

/// Iterator over entries in [`RecordHardLinkStorage`].
#[derive(derive_more::Debug)]
#[debug(bound())]
#[debug("Iter(..)")]
pub struct Iter<'a, Size>(DashIter<'a, u64, (Size, Vec<PathBuf>)>);

/// [Item](Iterator::Item) of [`Iter`].
#[derive(derive_more::Debug)]
#[debug(bound())]
#[debug("IterItem(..)")]
pub struct IterItem<'a, Size>(RefMulti<'a, u64, (Size, Vec<PathBuf>)>);

impl<'a, Size> Iterator for Iter<'a, Size> {
    type Item = IterItem<'a, Size>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(IterItem)
    }
}

impl<'a, Size> IterItem<'a, Size> {
    /// Number of the inode.
    pub fn ino(&self) -> u64 {
        *self.0.key()
    }

    /// Size of the inode.
    pub fn size(&self) -> &Size {
        &self.0.value().0
    }

    /// Links of the inode.
    pub fn links(&self) -> &[PathBuf] {
        &self.0.value().1
    }
}
