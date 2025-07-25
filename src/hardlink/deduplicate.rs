use crate::{data_tree::DataTree, os_string_display::OsStringDisplay, size};

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
        data_tree: &mut DataTree<OsStringDisplay, Size>,
    ) -> Result<Self::Report, Self::Error>;
}

/// Do deduplicate the sizes of hardlinks.
#[cfg(unix)]
pub type Do<Size> = super::HardlinkAware<Size>;
/// Do not deduplicate the sizes of hardlinks.
pub type DoNot = super::HardlinkIgnorant;
