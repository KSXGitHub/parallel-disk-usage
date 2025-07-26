use super::size::Bytes;
use std::fs::Metadata;

#[cfg(unix)]
use super::size::Blocks;
#[cfg(unix)]
use std::os::unix::prelude::MetadataExt;

/// Infers size from a [`Metadata`].
pub trait GetSize {
    type Size;
    fn get_size(&self, metadata: &Metadata) -> Self::Size;
}

/// Returns [`metadata.len()`](Metadata::len).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetApparentSize;
impl GetSize for GetApparentSize {
    type Size = Bytes;
    #[inline]
    fn get_size(&self, metadata: &Metadata) -> Self::Size {
        metadata.len().into()
    }
}

/// Returns [`metadata.blocks() * 512`](Metadata::blksize) (POSIX only).
#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetBlockSize;
#[cfg(unix)]
impl GetSize for GetBlockSize {
    type Size = Bytes;
    #[inline]
    fn get_size(&self, metadata: &Metadata) -> Self::Size {
        (metadata.blocks() * 512).into()
    }
}

/// Returns [`metadata.blocks()`](Metadata::blocks) (POSIX only).
#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetBlockCount;
#[cfg(unix)]
impl GetSize for GetBlockCount {
    type Size = Blocks;
    #[inline]
    fn get_size(&self, metadata: &Metadata) -> Self::Size {
        metadata.blocks().into()
    }
}
