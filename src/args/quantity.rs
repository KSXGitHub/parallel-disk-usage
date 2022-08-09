#[cfg(feature = "cli")]
use clap::ValueEnum;

/// Quantity to be measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum Quantity {
    /// Measure apparent sizes, equivalent to the
    /// [len](std::fs::Metadata::len) method.
    #[cfg_attr(feature = "cli", clap(name = "len"))]
    ApparentSize,
    /// Measure block sizes, equivalent to the
    /// [blksize](std::os::unix::prelude::MetadataExt::blksize) method (POSIX only).
    #[cfg(unix)]
    #[cfg_attr(feature = "cli", clap(name = "blksize"))]
    BlockSize,
    /// Count numbers of blocks, equivalent to the
    /// [blocks](std::os::unix::prelude::MetadataExt::blocks) method (POSIX only).
    #[cfg(unix)]
    #[cfg_attr(feature = "cli", clap(name = "blocks"))]
    BlockCount,
}
