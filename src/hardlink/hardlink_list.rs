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
use std::{fmt::Debug, path::Path};

#[cfg_attr(not(unix), cfg(expect(unused)))]
use pipe_trait::Pipe;

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
mod tests {
    use super::{AddError, HardlinkList, NumberOfLinksConflictError, SizeConflictError};
    use crate::size::Bytes;
    use pipe_trait::Pipe;
    use pretty_assertions::{assert_eq, assert_ne};

    const TABLE: &[(u64, u64, u64, &str)] = &[
        (241, 3652, 1, "a"),
        (569, 2210, 1, "b"),
        (110, 2350, 3, "c"),
        (110, 2350, 3, "c1"),
        (778, 1110, 1, "d"),
        (274, 6060, 2, "e"),
        (274, 6060, 2, "e1"),
        (883, 4530, 1, "f"),
    ];

    fn add<const ROW: usize>(list: HardlinkList<Bytes>) -> HardlinkList<Bytes> {
        let values = TABLE[ROW];
        let (ino, size, links, path) = values;
        if let Err(error) = list.add(ino.into(), size.into(), links, path.as_ref()) {
            panic!("Failed to add {values:?} (index: {ROW}) to the list: {error}");
        }
        list
    }

    #[test]
    fn insertion_order_is_irrelevant_to_equality() {
        let a = HardlinkList::new()
            .pipe(add::<3>)
            .pipe(add::<1>)
            .pipe(add::<4>)
            .pipe(add::<6>)
            .pipe(add::<5>)
            .pipe(add::<0>)
            .pipe(add::<7>)
            .pipe(add::<2>)
            .into_reflection();

        let b = HardlinkList::new()
            .pipe(add::<5>)
            .pipe(add::<6>)
            .pipe(add::<2>)
            .pipe(add::<0>)
            .pipe(add::<1>)
            .pipe(add::<3>)
            .pipe(add::<7>)
            .pipe(add::<4>)
            .into_reflection();

        let c = HardlinkList::new()
            .pipe(add::<0>)
            .pipe(add::<1>)
            .pipe(add::<2>)
            .pipe(add::<3>)
            .pipe(add::<4>)
            .pipe(add::<5>)
            .pipe(add::<6>)
            .pipe(add::<7>)
            .into_reflection();

        assert_eq!(a, b);
        assert_eq!(b, c);
        assert_eq!(a, c);
    }

    #[test]
    fn omitting_insertion_cause_inequality() {
        let a = HardlinkList::new()
            .pipe(add::<0>)
            .pipe(add::<1>)
            .pipe(add::<2>)
            .pipe(add::<3>)
            .pipe(add::<4>)
            .pipe(add::<5>)
            .pipe(add::<6>)
            .pipe(add::<7>)
            .into_reflection();

        let b = HardlinkList::new()
            .pipe(add::<0>)
            .pipe(add::<1>)
            .pipe(add::<2>)
            .pipe(add::<3>)
            .pipe(add::<4>)
            .pipe(add::<5>)
            .pipe(add::<7>)
            .into_reflection();

        assert_ne!(a, b);
        assert_ne!(b, a);
    }

    #[test]
    fn insertion_difference_cause_inequality() {
        let a = HardlinkList::new()
            .pipe(add::<0>)
            .pipe(add::<1>)
            .pipe(add::<2>)
            .pipe(add::<3>)
            .pipe(add::<4>)
            .pipe(add::<5>)
            .pipe(add::<6>)
            .into_reflection();

        let b = HardlinkList::new()
            .pipe(add::<0>)
            .pipe(add::<1>)
            .pipe(add::<2>)
            .pipe(add::<3>)
            .pipe(add::<4>)
            .pipe(add::<5>)
            .pipe(add::<7>)
            .into_reflection();

        assert_ne!(a, b);
        assert_ne!(b, a);
    }

    #[test]
    fn detect_size_change() {
        let list = HardlinkList::<Bytes>::new();
        list.add(123.into(), 100.into(), 1, "a".as_ref())
            .expect("add the first path");
        let actual = list
            .add(123.into(), 110.into(), 1, "b".as_ref())
            .expect_err("add the second path");
        let expected = AddError::SizeConflict(SizeConflictError {
            ino: 123.into(),
            recorded: 100.into(),
            detected: 110.into(),
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn detect_number_of_links_change() {
        let list = HardlinkList::<Bytes>::new();
        list.add(123.into(), 100.into(), 1, "a".as_ref())
            .expect("add the first path");
        let actual = list
            .add(123.into(), 100.into(), 2, "b".as_ref())
            .expect_err("add the second path");
        let expected = AddError::NumberOfLinksConflict(NumberOfLinksConflictError {
            ino: 123.into(),
            recorded: 1,
            detected: 2,
        });
        assert_eq!(actual, expected);
    }
}
