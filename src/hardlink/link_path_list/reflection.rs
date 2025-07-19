use super::LinkPathList;
use derive_more::{From, Into, IntoIterator};
use pipe_trait::Pipe;
use std::{collections::HashSet, path::PathBuf};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Intermediate format used for construction and inspection of
/// [`LinkPathList`]'s internal content.
///
/// **Equality:** `Reflection` implements `PartialEq` and `Eq` traits.
///
/// **Serialization and deserialization:** _(feature: `json`)_ `Reflection` implements
/// `Serialize` and `Deserialize` traits, this allows functions in `serde_json` to convert
/// a `Reflection` into/from JSON.
#[derive(Debug, Default, Clone, PartialEq, Eq, From, Into, IntoIterator)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct Reflection(pub HashSet<PathBuf>);

impl Reflection {
    /// Create an empty reflection.
    pub fn new() -> Self {
        Reflection::default()
    }

    /// Get the number of paths in the reflection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check whether the reflection has any path.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<LinkPathList> for Reflection {
    fn from(value: LinkPathList) -> Self {
        value.0.into_iter().collect::<HashSet<_>>().pipe(Reflection)
    }
}

impl From<Reflection> for LinkPathList {
    fn from(value: Reflection) -> Self {
        value.0.into_iter().collect::<Vec<_>>().pipe(LinkPathList)
    }
}
