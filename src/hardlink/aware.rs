use super::{RecordHardlinks, RecordHardlinksArgument};
use crate::{
    hardlink::HardlinkList,
    inode::InodeNumber,
    reporter::{event::HardlinkDetection, Event, Reporter},
    size,
};
use derive_more::{AsMut, AsRef, From, Into};
use pipe_trait::Pipe;
use smart_default::SmartDefault;
use std::{fmt::Debug, os::unix::fs::MetadataExt};

/// Be aware of hardlinks. Treat them as links that share space.
/// Detect files with more than 1 links and record them.
/// Deduplicate them (remove duplicated size) from total size to
/// accurately reflect the real size of their containers.
#[derive(Debug, SmartDefault, Clone, AsRef, AsMut, From, Into)]
pub struct Aware<Size> {
    /// Map an inode number to its size and detected paths.
    record: HardlinkList<Size>,
}

pub use Aware as HardlinkAware;

impl<Size> Aware<Size> {
    /// Create new hardlinks handler.
    pub fn new() -> Self {
        HardlinkList::default().pipe(Aware::from)
    }

    /// Create a detector/recorder of hardlinks.
    pub fn from_record(record: HardlinkList<Size>) -> Self {
        Aware::from(record)
    }
}

impl<Size, Report> RecordHardlinks<Size, Report> for Aware<Size>
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
