use derive_more::{Display, From, Into, LowerHex, Octal, UpperHex};

/// The inode number of a file or directory.
#[derive(
    Debug, Display, LowerHex, UpperHex, Octal, Clone, Copy, PartialEq, Eq, Hash, From, Into,
)]
pub struct InodeNumber(u64);

/// POSIX-exclusive functions.
#[cfg(unix)]
impl InodeNumber {
    /// Get inode number of a [`std::fs::Metadata`].
    pub fn get(stats: &std::fs::Metadata) -> Self {
        use pipe_trait::Pipe;
        use std::os::unix::fs::MetadataExt;
        stats.ino().pipe(InodeNumber)
    }
}
