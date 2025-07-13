use super::LinkPathList;
use derive_more::{From, Into, IntoIterator};
use pipe_trait::Pipe;
use std::{collections::HashSet, path::PathBuf};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Intermediate format used for construction and inspection of
/// [`LinkPathList`]'s internal content.
///
/// **Serialization and deserialization:** _(feature: `json`)_ `Reflection` implements
/// `Serialize` and `Deserialize` traits, this allows functions in `serde_json` to convert
/// a `Reflection` into/from JSON.
#[derive(Debug, Clone, PartialEq, Eq, From, Into, IntoIterator)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct Reflection(pub HashSet<PathBuf>);

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
