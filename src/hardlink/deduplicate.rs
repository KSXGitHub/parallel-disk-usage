use crate::{data_tree::DataTree, os_string_display::OsStringDisplay, size};

// TODO:
// Consider changing `deduplicate` into one that transforms `HardlinkDuplicated<DataTree>` into `DataTree`.
// `HardlinkDuplicated` (name is non-final) cannot be constructed manually and is the only type accepted by `deduplicate`.

type DataTreeTuple<Size, Other> = (DataTree<OsStringDisplay, Size>, Other);

/// Result type of [`DeduplicateSharedSize::deduplicate`].
pub type Result<Size, Report, Error> =
    std::result::Result<DataTreeTuple<Size, Report>, DataTreeTuple<Size, Error>>;

/// Ability to correct the sizes in a [`DataTree`] by reducing the size of recorded shared links.
///
/// The input tree is assumed to be not yet deduplicated.
pub trait DeduplicateSharedSize<Size: size::Size>: Sized {
    /// Report returned when [`DeduplicateSharedSize::deduplicate`] succeeds.
    type Report;
    /// Error returned when [`DeduplicateSharedSize::deduplicate`] fails.
    type Error;
    /// Correct the sizes in a [`DataTree`] by reducing the size of recorded shared links.
    fn deduplicate(
        self,
        data_tree: DataTree<OsStringDisplay, Size>,
    ) -> Result<Size, Self::Report, Self::Error>;
}

/// Do deduplicate the sizes of hardlinks.
pub type Do<Size> = super::HardlinkAware<Size>;
/// Do not deduplicate the sizes of hardlinks.
pub type DoNot = super::HardlinkIgnorant;
