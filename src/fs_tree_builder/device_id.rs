/// Unique identifier for a device or filesystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeviceId(Inner);

#[cfg(unix)]
type Inner = u64;

#[cfg(not(unix))]
type Inner = ();

/// Retrieve the [`DeviceId`] from filesystem metadata.
#[cfg(unix)]
pub(crate) fn get_device_id(stats: &std::fs::Metadata) -> DeviceId {
    use std::os::unix::fs::MetadataExt;
    DeviceId(stats.dev())
}

/// Retrieve the [`DeviceId`] from filesystem metadata.
///
/// On unsupported platforms, all entries share the same [`DeviceId`],
/// effectively disabling cross-device detection.
#[cfg(not(unix))]
pub(crate) fn get_device_id(_stats: &std::fs::Metadata) -> DeviceId {
    DeviceId(())
}
