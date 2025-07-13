pub mod binary_version;
pub mod schema_version;

pub use binary_version::BinaryVersion;
pub use schema_version::SchemaVersion;

use crate::{
    data_tree::DataTreeReflection,
    hardlink::HardlinkListReflection,
    size::{self, Blocks, Bytes},
};
use derive_more::{Deref, DerefMut, From, TryInto};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// The `"tree"` field of [`JsonData`].
#[derive(Debug, Clone, Deref, DerefMut)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "json", serde(rename_all = "kebab-case"))]
pub struct JsonTree<Size: size::Size> {
    /// The main data of the tree.
    #[deref]
    #[deref_mut]
    #[cfg_attr(feature = "json", serde(flatten))]
    pub data: DataTreeReflection<String, Size>,
    /// Optional list of shared inodes, their sizes, and their many links.
    #[cfg_attr(feature = "json", serde(skip_serializing_if = "Option::is_none"))]
    pub shared_inodes: Option<HardlinkListReflection<Size>>,
}

/// The `"unit"` field of [`JsonData`] and the root node of the tree.
#[derive(Debug, Clone, From, TryInto)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "json", serde(tag = "unit", content = "tree"))]
#[cfg_attr(feature = "json", serde(rename_all = "kebab-case"))]
pub enum UnitAndTree {
    /// Tree where size is [bytes](Bytes).
    Bytes(JsonTree<Bytes>),
    /// Tree where size is [blocks](Blocks).
    Blocks(JsonTree<Blocks>),
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
    pub unit_and_tree: UnitAndTree,
}
