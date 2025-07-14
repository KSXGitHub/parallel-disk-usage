use super::{
    DeduplicateSharedSize, HardlinkList, LinkPathList, RecordHardlinks, RecordHardlinksArgument,
};
use crate::{
    data_tree::DataTree,
    inode::InodeNumber,
    os_string_display::OsStringDisplay,
    reporter::{event::HardlinkDetection, Event, Reporter},
    size,
};
use derive_more::{AsMut, AsRef, From, Into};
use pipe_trait::Pipe;
use smart_default::SmartDefault;
use std::{convert::Infallible, fmt::Debug, os::unix::fs::MetadataExt, path::Path};

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

impl<Size> DeduplicateSharedSize<Size> for Aware<Size>
where
    DataTree<OsStringDisplay, Size>: Send,
    Size: size::Size + Sync,
{
    type Report = HardlinkList<Size>;
    type Error = Infallible;
    fn deduplicate(
        self,
        data_tree: &mut DataTree<OsStringDisplay, Size>,
    ) -> Result<Self::Report, Self::Error> {
        let record: Self::Report = self.into();
        let hardlink_info: Box<[(Size, LinkPathList)]> = record
            .iter()
            .map(|values| (*values.size(), values.links().clone()))
            .collect();
        let hardlink_info: Box<[(Size, Vec<&Path>)]> = hardlink_info
            .iter()
            .map(|(size, paths)| (*size, paths.iter().map(AsRef::as_ref).collect()))
            .collect();
        data_tree.par_deduplicate_hardlinks(&hardlink_info);
        Ok(record)
    }
}
