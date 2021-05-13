use super::ParsedValue;
use derive_more::Display;

/// The [`DisplayOutput`](Size::DisplayOutput) type of [`Bytes`].
#[derive(Debug, Display, Clone, Copy)]
pub enum Output {
    /// Display the value as-is.
    PlainNumber(u64),
    /// Display the value with unit a suffix.
    Units(ParsedValue),
}
