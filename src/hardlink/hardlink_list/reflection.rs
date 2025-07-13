use super::HardlinkList;
use crate::{
    hardlink::{LinkPathList, LinkPathListReflection},
    inode::InodeNumber,
};
use dashmap::DashMap;
use derive_more::{From, Into, IntoIterator};
use pipe_trait::Pipe;
use std::collections::HashMap;

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Intermediate format used for construction and inspection of
/// [`HardlinkList`]'s internal content.
///
/// **Serialization and deserialization:** _(feature: `json`)_ `Reflection` implements
/// `Serialize` and `Deserialize` traits, this allows functions in `serde_json` to convert
/// a `Reflection` into/from JSON.
#[derive(Debug, Clone, PartialEq, Eq, From, Into, IntoIterator)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct Reflection<Size>(pub HashMap<InodeNumber, Value<Size>>);

/// Size and list of link paths corresponding to an [`InodeNumber`] in [`Reflection`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct Value<Size> {
    pub size: Size,
    pub links: LinkPathListReflection,
}

impl<Size> Value<Size> {
    /// Create a new value.
    fn new(size: Size, links: LinkPathList) -> Self {
        let links = links.into();
        Value { size, links }
    }

    /// Convert the internal [`LinkPathListReflection`] into a [`LinkPathList`].
    fn into_list(self) -> (Size, LinkPathList) {
        let Value { size, links } = self;
        (size, links.into())
    }
}

impl<Size> From<HardlinkList<Size>> for Reflection<Size> {
    fn from(value: HardlinkList<Size>) -> Self {
        value
            .0
            .into_iter()
            .map(|(ino, (size, links))| (ino, Value::new(size, links)))
            .collect::<HashMap<_, _>>()
            .pipe(Reflection)
    }
}

impl<Size> From<Reflection<Size>> for HardlinkList<Size> {
    fn from(value: Reflection<Size>) -> Self {
        value
            .into_iter()
            .map(|(ino, value)| (ino, value.into_list()))
            .collect::<DashMap<_, _>>()
            .pipe(HardlinkList)
    }
}
