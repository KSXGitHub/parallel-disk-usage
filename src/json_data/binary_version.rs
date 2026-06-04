#![cfg_attr(
    dylint_lib = "perfectionist",
    expect(
        perfectionist::import_grouping,
        reason = "single_group cannot keep #[cfg]-gated imports in their own trailing group; see issue #436"
    )
)]

use derive_more::{AsMut, AsRef, From, FromStr, Into};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Version of the current `pdu` program.
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Version of the `pdu` program that created the input JSON.
#[derive(Debug, Clone, PartialEq, Eq, AsRef, AsMut, From, Into, FromStr)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct BinaryVersion(String);

impl BinaryVersion {
    /// Get version of the current `pdu` program as a [`BinaryVersion`].
    #[inline]
    pub fn current() -> Self {
        CURRENT_VERSION.to_string().into()
    }
}
