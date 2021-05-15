use super::size::Bytes;
use std::fs::Metadata;

#[cfg(unix)]
use super::size::Blocks;
#[cfg(unix)]
use std::os::unix::prelude::MetadataExt;

/// Infers size from a [`Metadata`].
pub type SizeGetter<Size> = fn(&Metadata) -> Size;
/// Returns [`metadata.len()`](Metadata::len).
pub const GET_APPARENT_SIZE: SizeGetter<Bytes> = |metadata| metadata.len().into();
/// Returns [`metadata.blksize()`](Metadata::blksize) (POSIX only).
#[cfg(unix)]
pub const GET_BLOCK_SIZE: SizeGetter<Bytes> = |metadata| metadata.blksize().into();
/// Returns [`metadata.blocks()`](Metadata::blocks) (POSIX only).
#[cfg(unix)]
pub const GET_BLOCK_COUNT: SizeGetter<Blocks> = |metadata| metadata.blocks().into();
