pub mod iter;
pub mod reflection;
pub mod summary;

pub use iter::Iter;
pub use reflection::Reflection;
pub use summary::Summary;

pub use Reflection as HardlinkListReflection;
pub use Summary as SharedLinkSummary;

use crate::{hardlink::LinkPathList, inode::InodeNumber, size};
use dashmap::DashMap;
use derive_more::{Display, Error};
use smart_default::SmartDefault;
use std::fmt::Debug;

#[cfg_attr(not(unix), cfg(test))]
use pipe_trait::Pipe;
#[cfg_attr(not(unix), cfg(test))]
use std::path::Path;

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
    /// Map an inode number to its size, number of links, and detected paths.
    DashMap<InodeNumber, Value<Size>>,
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

/// Error that occurs when a different size was detected for the same [`ino`][ino].
///
/// <!-- Should have been `std::os::unix::fs::MetadataExt::ino` but it would error on Windows -->
/// [ino]: https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html#tymethod.ino
#[derive(Debug, Display, Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[display(bound(Size: Debug))]
#[display("Size for inode {ino} changed from {recorded:?} to {detected:?}")]
pub struct SizeConflictError<Size> {
    pub ino: InodeNumber,
    pub recorded: Size,
    pub detected: Size,
}

/// Error that occurs when a different [`nlink`][nlink] was detected for the same [`ino`][ino].
///
/// <!-- Should have been `std::os::unix::fs::MetadataExt::ino` but it would error on Windows -->
/// [nlink]: https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html#tymethod.nlink
/// [ino]: https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html#tymethod.ino
#[derive(Debug, Display, Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[display("Number of links of inode {ino} changed from {recorded:?} to {detected:?}")]
pub struct NumberOfLinksConflictError {
    pub ino: InodeNumber,
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
    #[cfg_attr(not(unix), cfg(test))] // this function isn't used on non-POSIX except in tests
    pub(crate) fn add(
        &self,
        ino: InodeNumber,
        size: Size,
        links: u64,
        path: &Path,
    ) -> Result<(), AddError<Size>> {
        let mut assertions = Ok(());
        self.0
            .entry(ino)
            .and_modify(|recorded| {
                if size != recorded.size {
                    assertions = Err(AddError::SizeConflict(SizeConflictError {
                        ino,
                        recorded: recorded.size,
                        detected: size,
                    }));
                    return;
                }

                if links != recorded.links {
                    assertions = Err(AddError::NumberOfLinksConflict(
                        NumberOfLinksConflictError {
                            ino,
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
