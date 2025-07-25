use derive_more::{Display, Error};
use std::{convert::Infallible, process::ExitCode};

/// Error caused by the CLI program.
#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum RuntimeError {
    /// When it fails to write JSON representation of
    /// [DataTreeReflection](crate::data_tree::Reflection) to stdout.
    #[display("SerializationFailure: {_0}")]
    SerializationFailure(serde_json::Error),
    /// When it fails to read JSON representation of
    /// [DataTreeReflection](crate::data_tree::Reflection) from stdin.
    #[display("DeserializationFailure: {_0}")]
    DeserializationFailure(serde_json::Error),
    /// When `--json-input` and file names are both specified.
    #[display("JsonInputArgConflict: Arguments exist alongside --json-input")]
    JsonInputArgConflict,
    /// When input JSON data is not a valid tree.
    #[display("InvalidInputReflection: {_0}")]
    InvalidInputReflection(#[error(not(source))] String),
    /// When the user attempts to use unavailable platform-specific features.
    #[display("UnsupportedFeature: {_0}")]
    UnsupportedFeature(UnsupportedFeature),
}

/// Error caused by the user attempting to use unavailable platform-specific features.
#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum UnsupportedFeature {
    /// Using `--deduplicate-hardlinks` on non-POSIX.
    #[cfg(not(unix))]
    #[display("Feature --deduplicate-hardlinks is not available on this platform")]
    DeduplicateHardlink,
}

impl From<Infallible> for RuntimeError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

impl RuntimeError {
    /// Convert error into exit code.
    pub fn code(&self) -> ExitCode {
        ExitCode::from(match self {
            RuntimeError::SerializationFailure(_) => 2,
            RuntimeError::DeserializationFailure(_) => 3,
            RuntimeError::JsonInputArgConflict => 4,
            RuntimeError::InvalidInputReflection(_) => 5,
            RuntimeError::UnsupportedFeature(_) => 6,
        })
    }
}
