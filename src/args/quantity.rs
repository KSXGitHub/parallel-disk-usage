#[cfg(feature = "cli")]
use clap::ValueEnum;

/// Quantity to be measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum Quantity {
    /// Measure apparent sizes.
    #[cfg_attr(feature = "cli", clap(alias = "len"))]
    ApparentSize,
    /// Measure block sizes (block-count * 512B).
    #[cfg(unix)]
    #[cfg_attr(feature = "cli", clap(alias = "blksize"))]
    BlockSize,
    /// Count numbers of blocks.
    #[cfg(unix)]
    #[cfg_attr(feature = "cli", clap(alias = "blocks"))]
    BlockCount,
}

impl Quantity {
    /// Default value of the `--quantity` flag.
    #[cfg(unix)]
    pub(crate) const DEFAULT: Self = Quantity::BlockSize;
    /// Default value of the `--quantity` flag.
    #[cfg(not(unix))]
    pub(crate) const DEFAULT: Self = Quantity::ApparentSize;
}
