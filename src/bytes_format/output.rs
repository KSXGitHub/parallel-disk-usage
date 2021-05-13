use super::ParsedValue;
use derive_more::Display;

/// The [`DisplayOutput`](crate::size::Size::DisplayOutput) type of [`Bytes`](crate::size::Bytes).
#[derive(Debug, Display, Clone, Copy)]
pub enum Output {
    /// Display the value as-is.
    PlainNumber(u64),
    /// Display the value with unit a suffix.
    Units(ParsedValue),
}
