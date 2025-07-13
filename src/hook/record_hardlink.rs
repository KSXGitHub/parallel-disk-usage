pub mod storage;

pub use storage::HardlinkList;

use super::{Hook, HookArgument};
use crate::{
    inode::InodeNumber,
    reporter::{event::HardlinkDetection, Event, Reporter},
    size,
};
use std::{fmt::Debug, os::unix::fs::MetadataExt};

/// A [hook](Hook) that record files with more than 1 links.
#[derive(Debug, Clone, Copy)]
pub struct RecordHardlink<'a, Size> {
    /// Map an inode number to its size and detected paths.
    storage: &'a HardlinkList<Size>,
}

impl<'a, Size> RecordHardlink<'a, Size> {
    /// Create a [hook](Hook) to record files with more than 1 links.
    pub fn new(storage: &'a HardlinkList<Size>) -> Self {
        RecordHardlink { storage }
    }
}

impl<'a, Size, Report> Hook<Size, Report> for RecordHardlink<'a, Size>
where
    Size: size::Size + Eq + Debug,
    Report: Reporter<Size> + ?Sized,
{
    fn run_hook(&self, argument: HookArgument<Size, Report>) {
        let HookArgument {
            path,
            stats,
            size,
            reporter,
        } = argument;

        if stats.is_dir() {
            return;
        }

        let links = stats.nlink();
        if links <= 1 {
            return;
        }

        reporter.report(Event::DetectHardlink(HardlinkDetection {
            path,
            stats,
            size,
            links,
        }));

        let ino = InodeNumber::get(stats);
        self.storage.add(ino, size, path).unwrap(); // TODO: propagate the error
    }
}
