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
pub enum UnitAndTree {
    /// Tree where data is [bytes](Bytes).
    Bytes(Reflection<String, Bytes>),
    /// Tree where data is [blocks](Blocks).
    Blocks(Reflection<String, Blocks>),
}

/// Output of the program with `--json-output` flag as well as
/// input of the program with `--json-input` flag.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct JsonData {
    /// The `"unit"` field and the `"tree"` field.
    #[cfg_attr(feature = "json", serde(flatten))]
    pub unit_and_tree: UnitAndTree,
}
