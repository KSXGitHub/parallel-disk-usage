/// ANSI color style for terminal output.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Color {
    /// ANSI escape sequence prefix to apply before the colored text.
    pub ansi_prefix: String,
}

impl Color {
    /// The ANSI escape sequence prefix for the default directory color (bold blue).
    pub const DIRECTORY_ANSI_PREFIX: &'static str = "\x1b[1;34m";

    /// The ANSI reset sequence.
    pub const RESET: &'static str = "\x1b[0m";

    /// Create a [`Color`] with the given ANSI prefix.
    pub fn new(ansi_prefix: impl Into<String>) -> Self {
        Color {
            ansi_prefix: ansi_prefix.into(),
        }
    }

    /// The default directory color (bold blue).
    pub fn directory() -> Self {
        Color::new(Self::DIRECTORY_ANSI_PREFIX)
    }
}
