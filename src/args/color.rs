#[cfg(feature = "cli")]
use clap::ValueEnum;

/// When to use colors in the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum ColorWhen {
    /// Detect whether the output is a terminal and use colors accordingly.
    #[default]
    Auto,
    /// Always use colors.
    Always,
    /// Never use colors.
    Never,
}
