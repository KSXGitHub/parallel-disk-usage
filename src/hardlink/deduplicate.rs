use crate::{data_tree::DataTree, os_string_display::OsStringDisplay, size};

// TODO:
// Problem: It is possible for the users to call deduplicate twice on 1 data tree, which is unsound.
// Potential solution #1: Consider adding the data tree to the deduplication record as hidden field until it is processed.
// Potential solution #2: Perhaps the deduplication should be done in FsTreeBuilder itself.

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
pub type Do<Size> = super::HardlinkAware<Size>;
/// Do not deduplicate the sizes of hardlinks.
pub type DoNot = super::HardlinkIgnorant;
