//! Readers of the filesystem metadata of a path.

use pipe_trait::Pipe;
use std::fs::symlink_metadata;
use std::path::Path;

/// Read [apparent size](std::fs::Metadata::len) of a path.
pub fn read_apparent_size(path: &Path) -> u64 {
    path.pipe(symlink_metadata)
        .unwrap_or_else(|error| panic!("Can't read metadata at {path:?}: {error}"))
        .len()
}

/// Read [ino](std::os::unix::fs::MetadataExt::ino) of a path.
#[cfg(unix)]
pub fn read_inode_number(path: &Path) -> u64 {
    use std::os::unix::fs::MetadataExt;
    path.pipe(symlink_metadata)
        .unwrap_or_else(|error| panic!("Can't read metadata at {path:?}: {error}"))
        .ino()
}
