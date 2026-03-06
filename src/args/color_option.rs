#[cfg(feature = "cli")]
use clap::ValueEnum;

/// When to colorize the output.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum ColorOption {
    /// Colorize output only when stdout is a terminal.
    #[default]
    Auto,
    /// Always colorize the output.
    Always,
    /// Never colorize the output.
    Never,
}
