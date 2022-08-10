#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "json")]
use std::convert::TryFrom;
#[cfg(feature = "json")]
use thiserror::Error;

/// Content of [`SchemaVersion`].
pub const SCHEMA_VERSION: &str = "2021-06-05";

/// Verifying schema version.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "json", serde(try_from = "String", into = "&str"))]
pub struct SchemaVersion;

/// Error when trying to parse [`SchemaVersion`].
#[cfg(feature = "json")]
#[derive(Debug, Error)]
#[error("InvalidSchema: {:?}: input schema is not {:?}", input, SCHEMA_VERSION)]
pub struct InvalidSchema {
    /// The input string.
    pub input: String,
}

#[cfg(feature = "json")]
impl TryFrom<String> for SchemaVersion {
    type Error = InvalidSchema;
    fn try_from(input: String) -> Result<Self, Self::Error> {
        if input == SCHEMA_VERSION {
            Ok(SchemaVersion)
        } else {
            Err(InvalidSchema { input })
        }
    }
}

impl<'a> From<SchemaVersion> for &'a str {
    fn from(_: SchemaVersion) -> Self {
        SCHEMA_VERSION
    }
}
