use super::HardlinkList;
use crate::{
    hardlink::{LinkPathList, LinkPathListReflection},
    inode::InodeNumber,
};
use dashmap::DashMap;
use derive_more::{Display, Error, From, Into, IntoIterator};
use pipe_trait::Pipe;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Intermediate format used for construction and inspection of
/// [`HardlinkList`]'s internal content.
///
/// **Serialization and deserialization:** _(feature: `json`)_ `Reflection` implements
/// `Serialize` and `Deserialize` traits, this allows functions in `serde_json` to convert
/// a `Reflection` into/from JSON.
#[derive(Debug, Clone, PartialEq, Eq, From, Into, IntoIterator)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct Reflection<Size>(pub Vec<ReflectionEntry<Size>>);

/// An entry in [`Reflection`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct ReflectionEntry<Size> {
    pub ino: InodeNumber,
    pub size: Size,
    pub links: LinkPathListReflection,
}

impl<Size> ReflectionEntry<Size> {
    /// Create a new value.
    fn new(ino: InodeNumber, size: Size, links: LinkPathList) -> Self {
        let links = links.into();
        ReflectionEntry { ino, size, links }
    }
}

impl<Size> From<HardlinkList<Size>> for Reflection<Size> {
    fn from(HardlinkList(list): HardlinkList<Size>) -> Self {
        list.into_iter()
            .map(|(ino, (size, links))| ReflectionEntry::new(ino, size, links))
            .collect::<Vec<_>>()
            .pipe(Reflection)
    }
}

/// Error that occurs when an attempt to convert a [`Reflection`] into a
/// [`HardlinkList`] fails.
#[derive(Debug, Display, Error, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConversionError {
    /// When the source has duplicated inode numbers.
    #[display("Inode number {_0} is duplicated")]
    DuplicatedInode(#[error(not(source))] InodeNumber),
}

impl<Size> TryFrom<Reflection<Size>> for HardlinkList<Size> {
    type Error = ConversionError;
    fn try_from(Reflection(entries): Reflection<Size>) -> Result<Self, Self::Error> {
        let map = DashMap::with_capacity(entries.len());

        for ReflectionEntry { ino, size, links } in entries {
            let links = links.into();
            if map.insert(ino, (size, links)).is_some() {
                return ino.pipe(ConversionError::DuplicatedInode).pipe(Err);
            }
        }

        map.pipe(HardlinkList).pipe(Ok)
    }
}
