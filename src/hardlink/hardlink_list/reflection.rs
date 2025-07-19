use super::{HardlinkList, Value};
use crate::{hardlink::LinkPathListReflection, inode::InodeNumber};
use dashmap::DashMap;
use derive_more::{Display, Error, Into, IntoIterator};
use pipe_trait::Pipe;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Intermediate format used for construction and inspection of
/// [`HardlinkList`]'s internal content.
///
/// **Guarantees:**
/// * Every inode number is unique.
/// * The internal list is always sorted by inode numbers.
///
/// **Equality:** `Reflection` implements `PartialEq` and `Eq` traits.
///
/// **Serialization and deserialization:** _(feature: `json`)_ `Reflection` implements
/// `Serialize` and `Deserialize` traits, this allows functions in `serde_json` to convert
/// a `Reflection` into/from JSON.
#[derive(Debug, Clone, PartialEq, Eq, Into, IntoIterator)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct Reflection<Size>(Vec<ReflectionEntry<Size>>);

impl<Size> Reflection<Size> {
    /// Get the number of entries inside the reflection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check whether the reflection has any entry.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate over the entries.
    pub fn iter(&self) -> impl Iterator<Item = &ReflectionEntry<Size>> + Clone {
        self.0.iter()
    }
}

/// An entry in [`Reflection`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct ReflectionEntry<Size> {
    /// The inode number of the file.
    pub ino: InodeNumber,
    /// Size of the file.
    pub size: Size,
    /// Total number of links of the file, both listed (in [`Self::paths`]) and unlisted.
    pub links: u64,
    /// Paths to the detected links of the file.
    pub paths: LinkPathListReflection,
}

impl<Size> ReflectionEntry<Size> {
    /// Create a new entry.
    fn new(ino: InodeNumber, Value { size, links, paths }: Value<Size>) -> Self {
        let paths = paths.into();
        ReflectionEntry {
            ino,
            size,
            links,
            paths,
        }
    }

    /// Dissolve [`ReflectionEntry`] into a pair of [`InodeNumber`] and [`Value`].
    fn dissolve(self) -> (InodeNumber, Value<Size>) {
        let ReflectionEntry {
            ino,
            size,
            links,
            paths,
        } = self;
        let paths = paths.into();
        (ino, Value { size, links, paths })
    }
}

impl<Size> From<Vec<ReflectionEntry<Size>>> for Reflection<Size> {
    /// Sort the list by inode numbers, then create the reflection.
    fn from(mut list: Vec<ReflectionEntry<Size>>) -> Self {
        list.sort_by_key(|entry| u64::from(entry.ino));
        Reflection(list)
    }
}

impl<Size> From<HardlinkList<Size>> for Reflection<Size> {
    fn from(HardlinkList(list): HardlinkList<Size>) -> Self {
        list.into_iter()
            .map(|(ino, value)| ReflectionEntry::new(ino, value))
            .collect::<Vec<_>>()
            .pipe(Reflection::from)
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

        for entry in entries {
            let (ino, value) = entry.dissolve();
            if map.insert(ino, value).is_some() {
                return ino.pipe(ConversionError::DuplicatedInode).pipe(Err);
            }
        }

        map.pipe(HardlinkList).pipe(Ok)
    }
}
