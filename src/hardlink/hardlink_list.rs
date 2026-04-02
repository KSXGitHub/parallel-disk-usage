pub mod iter;
pub mod reflection;
pub mod summary;

pub use iter::Iter;
pub use reflection::Reflection;
pub use summary::Summary;

pub use Reflection as HardlinkListReflection;
pub use Summary as SharedLinkSummary;

use crate::{device::DeviceNumber, hardlink::LinkPathList, inode::InodeNumber, size};
use dashmap::DashMap;
use derive_more::{Display, Error};
use smart_default::SmartDefault;
use std::fmt::Debug;

#[cfg(any(unix, test))]
use pipe_trait::Pipe;
#[cfg(any(unix, test))]
use std::path::Path;

/// Internal key used to uniquely identify an inode across all filesystems.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct InodeKey {
    /// Inode number within the device.
    ino: InodeNumber,
    /// Device number of the filesystem the inode belongs to.
    dev: DeviceNumber,
}

/// Map value in [`HardlinkList`].
#[derive(Debug, Clone)]
struct Value<Size> {
    /// The size of the file.
    size: Size,
    /// Total number of links of the file, both listed (in [`Self::paths`]) and unlisted.
    links: u64,
    /// Paths to the detected links of the file.
    paths: LinkPathList,
}

/// Storage to be used by [`crate::hardlink::RecordHardlinks`].
///
/// **Reflection:** `HardlinkList` does not implement `PartialEq`, `Eq`,
/// `Deserialize`, and `Serialize` directly. Instead, it can be converted into a
/// [`Reflection`] which implement these traits.
#[derive(Debug, SmartDefault, Clone)]
pub struct HardlinkList<Size>(
    /// Map an inode key (device + inode number) to its size, number of links, and detected paths.
    DashMap<InodeKey, Value<Size>>,
);

impl<Size> HardlinkList<Size> {
    /// Create a new record.
    pub fn new() -> Self {
        HardlinkList::default()
    }

    /// Get the number of entries in the list.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Create reflection.
    pub fn into_reflection(self) -> Reflection<Size> {
        self.into()
    }
}

/// Error that occurs when a different size was detected for the same [`ino`] and [`dev`].
///
/// [`ino`]: std::os::unix::fs::MetadataExt::ino
/// [`dev`]: std::os::unix::fs::MetadataExt::dev
#[derive(Debug, Display, Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[display(bound(Size: Debug))]
#[display("Size for inode {ino} on device {dev} changed from {recorded:?} to {detected:?}")]
pub struct SizeConflictError<Size> {
    pub ino: InodeNumber,
    pub dev: DeviceNumber,
    pub recorded: Size,
    pub detected: Size,
}

/// Error that occurs when a different [`nlink`] was detected for the same [`ino`] and [`dev`].
///
/// [`nlink`]: std::os::unix::fs::MetadataExt::nlink
/// [`ino`]: std::os::unix::fs::MetadataExt::ino
/// [`dev`]: std::os::unix::fs::MetadataExt::dev
#[derive(Debug, Display, Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[display(
    "Number of links of inode {ino} on device {dev} changed from {recorded:?} to {detected:?}"
)]
pub struct NumberOfLinksConflictError {
    pub ino: InodeNumber,
    pub dev: DeviceNumber,
    pub recorded: u64,
    pub detected: u64,
}

/// Error that occurs when it fails to add an item to [`HardlinkList`].
#[derive(Debug, Display, Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[display(bound(Size: Debug))]
#[non_exhaustive]
pub enum AddError<Size> {
    SizeConflict(SizeConflictError<Size>),
    NumberOfLinksConflict(NumberOfLinksConflictError),
}

impl<Size> HardlinkList<Size>
where
    Size: size::Size,
{
    /// Add an entry to the record.
    #[cfg(any(unix, test))] // this function isn't used on non-POSIX except in tests
    pub(crate) fn add(
        &self,
        ino: InodeNumber,
        dev: DeviceNumber,
        size: Size,
        links: u64,
        path: &Path,
    ) -> Result<(), AddError<Size>> {
        let key = InodeKey { ino, dev };
        let mut assertions = Ok(());
        self.0
            .entry(key)
            .and_modify(|recorded| {
                if size != recorded.size {
                    assertions = Err(AddError::SizeConflict(SizeConflictError {
                        ino,
                        dev,
                        recorded: recorded.size,
                        detected: size,
                    }));
                    return;
                }

                if links != recorded.links {
                    assertions = Err(AddError::NumberOfLinksConflict(
                        NumberOfLinksConflictError {
                            ino,
                            dev,
                            recorded: recorded.links,
                            detected: links,
                        },
                    ));
                    return;
                }

                recorded.paths.add(path.to_path_buf());
            })
            .or_insert_with(|| {
                let paths = path.to_path_buf().pipe(LinkPathList::single);
                Value { size, links, paths }
            });
        assertions
    }
}

#[cfg(test)]
mod test;
