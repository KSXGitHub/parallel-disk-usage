use super::{Hook, HookArgument};
use dashmap::DashMap;
use std::{fmt::Debug, os::unix::fs::MetadataExt, path::PathBuf};

/// Map an inode number to its size and detected paths.
type RecordHardLinkStorage<Size> = DashMap<u64, (Size, Vec<PathBuf>)>; // TODO: benchmark against Mutex<HashMap<u64, (Size, Vec<PathBuf>)>>

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

impl<'a, Size: Eq + Debug> Hook<Size> for RecordHardLink<'a, Size> {
    fn run_hook(&self, argument: HookArgument<Size>) {
        let HookArgument { path, stats, size } = argument;

        if stats.is_dir() || stats.nlink() <= 1 {
            return;
        }

        self.storage
            .entry(stats.ino())
            .and_modify(|(expected_size, paths)| {
                assert_eq!(
                    size, *expected_size,
                    "same ino but different sizes: {size:?} vs {expected_size:?}",
                );
                paths.push(path.to_path_buf());
            })
            .or_insert_with(|| (size, Vec::new()));
    }
}
