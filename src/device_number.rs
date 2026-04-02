use derive_more::{Display, From, Into, LowerHex, Octal, UpperHex};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// The device number of a filesystem.
#[derive(
    Debug, Display, LowerHex, UpperHex, Octal, Clone, Copy, PartialEq, Eq, Hash, From, Into,
)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct DeviceNumber(u64);

/// POSIX-exclusive functions.
#[cfg(unix)]
impl DeviceNumber {
    /// Get device number of a [`std::fs::Metadata`].
    #[inline]
    pub fn get(stats: &std::fs::Metadata) -> Self {
        use pipe_trait::Pipe;
        use std::os::unix::fs::MetadataExt;
        stats.dev().pipe(DeviceNumber)
    }
}
