use super::{HardlinkList, InodeKey, Value};
use crate::{hardlink::LinkPathListReflection, inode::InodeNumber};
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
/// * Every `(device, inode)` pair is unique within the scope of a single scan, but inode
///   numbers alone are **not** guaranteed to be unique: when scanning multiple filesystems,
///   two unrelated files on different devices can share the same inode number and will each
///   produce a separate entry. The reflection stores only the inode number (the JSON format
///   does not carry device information), so round-tripping a multi-filesystem scan through
///   JSON is an unsupported edge case.
/// * The internal list is always sorted by inode numbers (and by device number as a
///   tie-breaker when two entries share the same inode number).
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
    #[inline]
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
    fn from(list: Vec<ReflectionEntry<Size>>) -> Self {
        list.into_sorted_unstable_by_key(|entry| u64::from(entry.ino))
            .pipe(Reflection)
    }
}

impl<Size> From<HardlinkList<Size>> for Reflection<Size> {
    fn from(HardlinkList(list): HardlinkList<Size>) -> Self {
        // Collect to a vec, sort by (ino, dev) for a stable, deterministic order, then
        // strip dev before wrapping.  Sorting here (with dev still available) avoids the
        // nondeterminism that would arise from an unstable sort on ino alone when two
        // entries from different filesystems share the same inode number.
        let mut pairs: Vec<(InodeKey, Value<Size>)> = list.into_iter().collect();
        pairs.sort_unstable_by_key(|(key, _)| (u64::from(key.ino), key.dev));
        pairs
            .into_iter()
            .map(|(key, value)| ReflectionEntry::new(key.ino, value))
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

        for entry in entries {
            let (ino, value) = entry.dissolve();
            // Device number is unknown when loading from a reflection (e.g. JSON input);
            // use dev=0 as a placeholder. This means that when reloading JSON output that
            // was produced by scanning multiple filesystems, files from different devices
            // sharing the same inode number cannot be distinguished and therefore cannot
            // all be represented. Such duplicates cause a ConversionError::DuplicatedInode
            // and are treated as an unsupported edge case, since the JSON format does not
            // carry device information.
            let key = InodeKey { dev: 0, ino };
            if map.insert(key, value).is_some() {
                return ino.pipe(ConversionError::DuplicatedInode).pipe(Err);
            }
        }

        map.pipe(HardlinkList).pipe(Ok)
    }
}
