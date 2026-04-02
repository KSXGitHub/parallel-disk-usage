use super::{HardlinkList, InodeKey, Value};
use crate::{device_number::DeviceNumber, hardlink::LinkPathListReflection, inode::InodeNumber};
use dashmap::DashMap;
use derive_more::{Display, Error, Into, IntoIterator};
use into_sorted::IntoSortedUnstable;
use pipe_trait::Pipe;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Intermediate format used for construction and inspection of [`HardlinkList`]'s
/// internal content.
///
/// **Guarantees:**
/// * Every `(device, inode)` pair is unique.
/// * The internal list is always sorted by inode numbers (with device number as tie-breaker).
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
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check whether the reflection has any entry.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate over the entries.
    #[inline]
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
    /// Device number of the filesystem the inode belongs to.
    pub dev: DeviceNumber,
    /// Size of the file.
    pub size: Size,
    /// Total number of links of the file, both listed (in [`Self::paths`]) and unlisted.
    pub links: u64,
    /// Paths to the detected links of the file.
    pub paths: LinkPathListReflection,
}

impl<Size> ReflectionEntry<Size> {
    /// Create a new entry.
    #[inline]
    fn new(InodeKey { ino, dev }: InodeKey, Value { size, links, paths }: Value<Size>) -> Self {
        let paths = paths.into();
        ReflectionEntry {
            ino,
            dev,
            size,
            links,
            paths,
        }
    }

    /// Dissolve [`ReflectionEntry`] into a pair of [`InodeKey`] and [`Value`].
    #[inline]
    fn dissolve(self) -> (InodeKey, Value<Size>) {
        let ReflectionEntry {
            ino,
            dev,
            size,
            links,
            paths,
        } = self;
        let paths = paths.into();
        (InodeKey { ino, dev }, Value { size, links, paths })
    }
}

impl<Size> From<Vec<ReflectionEntry<Size>>> for Reflection<Size> {
    /// Sort the list by `(inode, device)`, then create the reflection.
    fn from(list: Vec<ReflectionEntry<Size>>) -> Self {
        list.into_sorted_unstable_by_key(|entry| (u64::from(entry.ino), u64::from(entry.dev)))
            .pipe(Reflection)
    }
}

impl<Size> From<HardlinkList<Size>> for Reflection<Size> {
    fn from(HardlinkList(list): HardlinkList<Size>) -> Self {
        list.into_iter()
            .map(|(key, value)| ReflectionEntry::new(key, value))
            .collect::<Vec<_>>()
            .pipe(Reflection::from)
    }
}

/// Error that occurs when an attempt to convert a [`Reflection`] into a
/// [`HardlinkList`] fails.
#[derive(Debug, Display, Error, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConversionError {
    /// When the source has a duplicated `(inode, device)` pair.
    #[display("Inode {ino} on device {dev} is duplicated")]
    DuplicatedInode {
        #[error(not(source))]
        ino: InodeNumber,
        #[error(not(source))]
        dev: DeviceNumber,
    },
}

impl ConversionError {
    fn duplicated_inode(InodeKey { ino, dev }: InodeKey) -> Self {
        ConversionError::DuplicatedInode { ino, dev }
    }
}

impl<Size> TryFrom<Reflection<Size>> for HardlinkList<Size> {
    type Error = ConversionError;
    fn try_from(Reflection(entries): Reflection<Size>) -> Result<Self, Self::Error> {
        let map = DashMap::with_capacity(entries.len());

        for entry in entries {
            let (key, value) = entry.dissolve();
            if map.insert(key, value).is_some() {
                return key.pipe(ConversionError::duplicated_inode).pipe(Err);
            }
        }

        map.pipe(HardlinkList).pipe(Ok)
    }
}
