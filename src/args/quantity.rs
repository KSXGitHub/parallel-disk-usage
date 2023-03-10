#[cfg(feature = "cli")]
use clap::ValueEnum;

/// Quantity to be measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum Quantity {
    /// Measure apparent sizes.
    ApparentSize,
    /// Measure block sizes (block-count * 512B).
    #[cfg(unix)]
    BlockSize,
    /// Count numbers of blocks.
    #[cfg(unix)]
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
