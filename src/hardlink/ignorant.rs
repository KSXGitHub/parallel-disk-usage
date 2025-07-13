use super::{RecordHardlinks, RecordHardlinksArgument};

/// Be ignorant of hardlinks. Treat them as real files.
/// Do not detect it. Do not deduplicate it.
/// Essentially no-op.
#[derive(Debug, Default, Clone, Copy)]
pub struct Ignorant;

pub use Ignorant as HardlinkIgnorant;

/// Do nothing to detect nor record any hardlink.
impl<Size, Reporter> RecordHardlinks<Size, Reporter> for Ignorant {
    /// Do nothing.
    fn record_hardlinks(&self, _: RecordHardlinksArgument<Size, Reporter>) {}
}
