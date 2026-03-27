/// Unique identifier for a device or filesystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId(Inner);

#[cfg(unix)]
type Inner = u64;

#[cfg(not(unix))]
type Inner = ();

/// Retrieve the [`DeviceId`] from filesystem metadata.
#[cfg(unix)]
pub fn get_device_id(stats: &std::fs::Metadata) -> DeviceId {
    use std::os::unix::fs::MetadataExt;
    DeviceId(stats.dev())
}

/// Retrieve the [`DeviceId`] from filesystem metadata.
///
/// On unsupported platforms, all entries share the same [`DeviceId`],
/// effectively disabling cross-device detection.
#[cfg(not(unix))]
pub fn get_device_id(_stats: &std::fs::Metadata) -> DeviceId {
    DeviceId(())
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::get_device_id;
    use std::fs::symlink_metadata;

    #[test]
    fn same_filesystem_returns_equal_ids() {
        let root_stats = symlink_metadata("/").expect("stat /");
        let root_stats2 = symlink_metadata("/").expect("stat / again");
        assert_eq!(
            get_device_id(&root_stats),
            get_device_id(&root_stats2),
            "same path should yield the same DeviceId",
        );
    }

    /// `/proc` is a virtual filesystem mounted separately from `/` on Linux.
    #[test]
    #[cfg(target_os = "linux")]
    fn different_filesystem_returns_different_ids() {
        let root_stats = symlink_metadata("/").expect("stat /");
        let proc_stats = symlink_metadata("/proc").expect("stat /proc");
        assert_ne!(
            get_device_id(&root_stats),
            get_device_id(&proc_stats),
            "/ and /proc should be on different devices",
        );
    }

    /// `/dev` is a separate filesystem (`devfs`) from `/` on macOS.
    #[test]
    #[cfg(target_os = "macos")]
    fn different_filesystem_returns_different_ids() {
        let root_stats = symlink_metadata("/").expect("stat /");
        let dev_stats = symlink_metadata("/dev").expect("stat /dev");
        assert_ne!(
            get_device_id(&root_stats),
            get_device_id(&dev_stats),
            "/ and /dev should be on different devices",
        );
    }
}
