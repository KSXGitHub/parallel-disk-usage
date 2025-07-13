pub mod binary_version;
pub mod schema_version;

pub use binary_version::BinaryVersion;
pub use schema_version::SchemaVersion;

use crate::{
    data_tree::Reflection,
    size::{Blocks, Bytes},
};
use derive_more::{From, TryInto};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// The `"unit"` field and the `"tree"` field of [`JsonData`].
#[derive(Debug, Clone, From, TryInto)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "json", serde(tag = "unit", content = "tree"))]
#[cfg_attr(feature = "json", serde(rename_all = "kebab-case"))]
pub enum JsonDataBody {
    /// Tree where size is [bytes](Bytes).
    Bytes(Reflection<String, Bytes>),
    /// Tree where size is [blocks](Blocks).
    Blocks(Reflection<String, Blocks>),
}

/// Output of the program with `--json-output` flag as well as
/// input of the program with `--json-input` flag.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "json", serde(rename_all = "kebab-case"))]
pub struct JsonData {
    /// The `"schema-version"` field.
    pub schema_version: SchemaVersion,
    /// The `"pdu"` field.
    #[cfg_attr(feature = "json", serde(rename = "pdu"))]
    pub binary_version: Option<BinaryVersion>,
    /// The `"unit"` field and the `"tree"` field.
    #[cfg_attr(feature = "json", serde(flatten))]
    pub body: JsonDataBody,
}
