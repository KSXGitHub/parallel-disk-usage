use std::{fs::Metadata, path::Path};

/// Argument to pass to [`Hook::run_hook`].
#[derive(Debug, Clone, Copy)]
pub struct HookArgument<'a, Size, Report: ?Sized> {
    pub path: &'a Path,
    pub stats: &'a Metadata,
    pub size: Size,
    pub reporter: &'a Report,
}

impl<'a, Size, Report: ?Sized> HookArgument<'a, Size, Report> {
    pub(crate) fn new(
        path: &'a Path,
        stats: &'a Metadata,
        size: Size,
        reporter: &'a Report,
    ) -> Self {
        HookArgument {
            path,
            stats,
            size,
            reporter,
        }
    }
}

/// Hook to run with a [`Path`] and its corresponding [`Metadata`] and size.
pub trait Hook<Size, Reporter: ?Sized> {
    fn run_hook(&self, argument: HookArgument<Size, Reporter>);
}

/// A [hook](Hook) that does nothing.
#[derive(Debug, Clone, Copy)]
pub struct DoNothing;
impl<Size, Reporter> Hook<Size, Reporter> for DoNothing {
    fn run_hook(&self, _: HookArgument<Size, Reporter>) {}
}

// `RecordHardlink` is POSIX-exclusive, because whilst Windows does have `MetadataExt::number_of_links`, it requires Nightly.
#[cfg(unix)]
pub mod record_hardlink;
#[cfg(unix)]
pub use record_hardlink::HardlinkList;
#[cfg(unix)]
pub use record_hardlink::RecordHardlink;
