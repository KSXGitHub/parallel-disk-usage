use derive_more::Display;

/// Error caused by the CLI program.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeError {
    /// Failed to get terminal width.
    #[display(fmt = "Failed to get terminal width")]
    TerminalWidthInferenceFailure,
}
