use std::{fs::Metadata, path::Path};

/// Argument to pass to [`RecordHardlinks::record_hardlinks`].
#[derive(Debug, Clone, Copy)]
pub struct Argument<'a, Size, Report: ?Sized> {
    pub path: &'a Path,
    pub stats: &'a Metadata,
    pub size: Size,
    pub reporter: &'a Report,
}

pub use Argument as RecordHardlinksArgument;

impl<'a, Size, Report: ?Sized> Argument<'a, Size, Report> {
    pub(crate) fn new(
        path: &'a Path,
        stats: &'a Metadata,
        size: Size,
        reporter: &'a Report,
    ) -> Self {
        Argument {
            path,
            stats,
            size,
            reporter,
        }
    }
}

/// Ability to detect and record hardlinks.
pub trait RecordHardlinks<Size, Reporter: ?Sized> {
    /// Error when [`RecordHardlinks::record_hardlinks`] fails.
    type Error;
    /// Perform hardlinks detection and recording.
    fn record_hardlinks(&self, argument: Argument<Size, Reporter>) -> Result<(), Self::Error>;
}

/// Do detect and record hardlinks.
pub type Do<Size> = super::HardlinkAware<Size>;
/// Do not detect nor record hardlinks.
pub type DoNot = super::HardlinkIgnorant;
