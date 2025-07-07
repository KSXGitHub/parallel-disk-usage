use std::{fs::Metadata, path::Path};

/// Argument to pass to [`Hook::run_hook`].
#[derive(Debug, Clone, Copy)]
pub struct HookArgument<'a, Size> {
    pub path: &'a Path,
    pub metadata: &'a Metadata,
    pub size: Size,
}

/// Hook to run with a [`Path`] and its corresponding [`Metadata`].
pub trait Hook<Size> {
    fn run_hook(&self, argument: HookArgument<Size>);
}

/// A [hook](Hook) that does nothing.
#[derive(Debug, Clone, Copy)]
pub struct DoNothing;
impl<Size> Hook<Size> for DoNothing {
    fn run_hook(&self, _: HookArgument<Size>) {}
}

// `RecordHardlink` is POSIX-exclusive, because whilst Windows does have `MetadataExt::number_of_links`, it requires Nightly.
#[cfg(unix)]
mod record_hardlink;
#[cfg(unix)]
pub use record_hardlink::*;
