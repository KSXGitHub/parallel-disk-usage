use derive_more::{Display, Error};

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
}
