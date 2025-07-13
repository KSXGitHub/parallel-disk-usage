use super::record::{RecordHardlinks, RecordHardlinksArgument};
use crate::{
    hardlink::HardlinkList,
    inode::InodeNumber,
    reporter::{event::HardlinkDetection, Event, Reporter},
    size,
};
use std::{fmt::Debug, os::unix::fs::MetadataExt};

/// Be aware of hardlinks. Treat them as links that share space.
/// Detect files with more than 1 links and record them.
/// Deduplicate them (remove duplicated size) from total size to
/// accurately reflect the real size of their containers.
#[derive(Debug, Clone, Copy)]
pub struct HardlinkAware<'a, Size> {
    /// Map an inode number to its size and detected paths.
    record: &'a HardlinkList<Size>,
}

impl<'a, Size> HardlinkAware<'a, Size> {
    /// Create a detector/recorder of hardlinks.
    pub fn new(record: &'a HardlinkList<Size>) -> Self {
        HardlinkAware { record }
    }
}

impl<'a, Size, Report> RecordHardlinks<Size, Report> for HardlinkAware<'a, Size>
where
    Size: size::Size + Eq + Debug,
    Report: Reporter<Size> + ?Sized,
{
    fn record_hardlinks(&self, argument: RecordHardlinksArgument<Size, Report>) {
        let RecordHardlinksArgument {
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
        self.record.add(ino, size, path).unwrap(); // TODO: propagate the error
    }
}
