use derive_more::{Display, From, Into, LowerHex, Octal, UpperHex};

#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

/// Whether to cross device boundary into a different filesystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceBoundary {
    Cross,
    Stay,
}

impl DeviceBoundary {
    /// Derive device boundary from `--one-file-system`.
    #[cfg(feature = "cli")]
    pub(crate) fn from_one_file_system(one_file_system: bool) -> Self {
        match one_file_system {
            false => DeviceBoundary::Cross,
            true => DeviceBoundary::Stay,
        }
    }
}

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
