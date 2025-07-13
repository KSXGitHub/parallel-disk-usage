use super::{Hook, HookArgument};
use crate::{
    reporter::{event::EncounterHardlink, Event, Reporter},
    size,
};
use dashmap::DashMap;
use std::{fmt::Debug, os::unix::fs::MetadataExt, path::PathBuf};

/// Map an inode number to its size and detected paths.
pub type RecordHardLinkStorage<Size> = DashMap<u64, (Size, Vec<PathBuf>)>; // TODO: benchmark against Mutex<HashMap<u64, (Size, Vec<PathBuf>)>>

/// A [hook](Hook) that record files with more than 1 links.
#[derive(Debug, Clone, Copy)]
pub struct RecordHardLink<'a, Size> {
    /// Map an inode number to its size and detected paths.
    storage: &'a RecordHardLinkStorage<Size>,
}

impl<'a, Size> RecordHardLink<'a, Size> {
    /// Create a [hook](Hook) to record files with more than 1 links.
    pub fn new(storage: &'a RecordHardLinkStorage<Size>) -> Self {
        RecordHardLink { storage }
    }
}

impl<'a, Size, Report> Hook<Size, Report> for RecordHardLink<'a, Size>
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

        reporter.report(Event::EncounterHardlink(EncounterHardlink {
            path,
            stats,
            size,
            links,
        }));

        self.storage
            .entry(stats.ino())
            .and_modify(|(expected_size, paths)| {
                assert_eq!(
                    size, *expected_size,
                    "same ino but different sizes: {size:?} vs {expected_size:?}",
                );
                paths.push(path.to_path_buf());
            })
            .or_insert_with(|| (size, vec![path.to_path_buf()]));
    }
}
