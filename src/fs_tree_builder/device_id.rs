/// Unique identifier for a device or filesystem.
///
/// Used to detect mount boundaries so that the tool does not traverse into
/// mounted filesystems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DeviceId(Inner);

#[cfg(unix)]
type Inner = u64;

#[cfg(windows)]
type Inner = Option<u32>;

#[cfg(not(any(unix, windows)))]
type Inner = ();

/// Retrieve the [`DeviceId`] from filesystem metadata.
#[cfg(unix)]
pub(super) fn get_device_id(stats: &std::fs::Metadata) -> DeviceId {
    use std::os::unix::fs::MetadataExt;
    DeviceId(stats.dev())
}

/// Retrieve the [`DeviceId`] from filesystem metadata.
#[cfg(windows)]
pub(super) fn get_device_id(stats: &std::fs::Metadata) -> DeviceId {
    use std::os::windows::fs::MetadataExt;
    DeviceId(stats.volume_serial_number())
}

/// Retrieve the [`DeviceId`] from filesystem metadata.
///
/// On unsupported platforms, all entries share the same [`DeviceId`],
/// effectively disabling cross-device detection.
#[cfg(not(any(unix, windows)))]
pub(super) fn get_device_id(_stats: &std::fs::Metadata) -> DeviceId {
    DeviceId(())
}
