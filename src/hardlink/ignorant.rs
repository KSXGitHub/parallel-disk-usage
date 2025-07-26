use super::{DeduplicateSharedSize, RecordHardlinks, RecordHardlinksArgument};
use crate::{data_tree::DataTree, os_string_display::OsStringDisplay, size};
use std::convert::Infallible;

/// Be ignorant of hardlinks. Treat them as real files.
/// Do not detect it. Do not deduplicate it.
/// Essentially no-op.
#[derive(Debug, Default, Clone, Copy)]
pub struct Ignorant;

pub use Ignorant as HardlinkIgnorant;

/// Do nothing to detect nor record any hardlink.
impl<Size, Reporter> RecordHardlinks<Size, Reporter> for Ignorant {
    /// Doing nothing cannot fail.
    type Error = Infallible;

    /// Do nothing.
    #[inline]
    fn record_hardlinks(
        &self,
        _: RecordHardlinksArgument<Size, Reporter>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Do nothing to deduplicate the sizes of hardlinks.
impl<Size> DeduplicateSharedSize<Size> for HardlinkIgnorant
where
    Size: size::Size + Sync,
{
    /// Return nothing.
    type Report = ();
    /// Doing nothing cannot fail.
    type Error = Infallible;

    /// Do nothing.
    #[inline]
    fn deduplicate(
        self,
        _: &mut DataTree<OsStringDisplay, Size>,
    ) -> Result<Self::Report, Self::Error> {
        Ok(())
    }
}
