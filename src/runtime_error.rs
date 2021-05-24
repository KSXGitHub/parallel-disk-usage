use derive_more::Display;

/// Error caused by the CLI program.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RuntimeError {}
