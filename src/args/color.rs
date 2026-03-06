#[cfg(feature = "cli")]
use clap::ValueEnum;

/// When to use colors in the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum ColorWhen {
    /// Detect if the output is a TTY and render colors accordingly.
    #[default]
    Auto,
    /// Always render colors.
    Always,
    /// Never render colors.
    Never,
}
