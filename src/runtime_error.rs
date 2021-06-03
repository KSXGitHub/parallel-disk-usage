use derive_more::Display;

/// Error caused by the CLI program.
#[derive(Debug, Display)]
#[non_exhaustive]
pub enum RuntimeError {
    /// When it fails to write JSON representation of
    /// [DataTreeReflection](crate::data_tree::Reflection) to stdout.
    #[display(fmt = "SerializationFailure: {}", _0)]
    SerializationFailure(serde_json::Error),
    /// When it fails to read JSON representation of
    /// [DataTreeReflection](crate::data_tree::Reflection) from stdin.
    #[display(fmt = "DeserializationFailure: {}", _0)]
    DeserializationFailure(serde_json::Error),
    /// When both `--json-input` and file names are both specified.
    #[display(fmt = "JsonInputArgConflict: Arguments exist alongside --json-input")]
    JsonInputArgConflict,
    /// When input JSON data is not a valid tree.
    #[display(fmt = "InvalidInputReflection: {}", _0)]
    InvalidInputReflection(String),
}
