use std::{fs::Metadata, path::Path};

/// Argument to pass to [`RecordHardlinks::record_hardlinks`].
#[derive(Debug, Clone, Copy)]
pub struct RecordHardlinksArgument<'a, Size, Report: ?Sized> {
    pub path: &'a Path,
    pub stats: &'a Metadata,
    pub size: Size,
    pub reporter: &'a Report,
}

impl<'a, Size, Report: ?Sized> RecordHardlinksArgument<'a, Size, Report> {
    pub(crate) fn new(
        path: &'a Path,
        stats: &'a Metadata,
        size: Size,
        reporter: &'a Report,
    ) -> Self {
        RecordHardlinksArgument {
            path,
            stats,
            size,
            reporter,
        }
    }
}

/// Ability to detect and record hardlinks.
pub trait RecordHardlinks<Size, Reporter: ?Sized> {
    /// Perform hardlinks detection and recording.
    fn record_hardlinks(&self, argument: RecordHardlinksArgument<Size, Reporter>);
}

/// Do detect and record hardlinks.
pub type Do<'a, Size> = super::HardlinkAware<'a, Size>;
/// Do not detect nor record hardlinks.
pub type DoNot = super::HardlinkIgnorant;
