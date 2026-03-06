#[cfg(feature = "cli")]
use clap::ValueEnum;

/// When to use colors in the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum ColorWhen {
    /// Use colors only when the output is a terminal.
    #[default]
    Auto,
    /// Always use colors.
    Always,
    /// Never use colors.
    Never,
}
