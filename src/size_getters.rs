use super::size::{Blocks, Bytes};
use std::fs::Metadata;

#[cfg(unix)]
use std::os::unix::prelude::MetadataExt;

/// Infers size from a [`Metadata`].
pub type SizeGetter<Size> = fn(&Metadata) -> Size;

/// Returns [`metadata.len()`](Metadata::len).
#[inline]
pub fn get_apparent_size(metadata: &Metadata) -> Bytes {
    metadata.len().into()
}

/// Returns [`metadata.blksize()`](Metadata::blksize) (POSIX only).
#[cfg(unix)]
#[inline]
pub fn get_block_size(metadata: &Metadata) -> Bytes {
    metadata.blksize().into()
}

/// Returns [`metadata.blocks()`](Metadata::blocks) (POSIX only).
#[cfg(unix)]
#[inline]
pub fn get_block_count(metadata: &Metadata) -> Blocks {
    metadata.blocks().into()
}
