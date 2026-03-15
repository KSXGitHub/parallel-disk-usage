use super::mount_point::find_mount_point;
use std::{
    ffi::OsStr,
    fs::canonicalize,
    io,
    path::{Path, PathBuf},
};
use sysinfo::{Disk, DiskKind};

#[cfg(target_os = "linux")]
use pipe_trait::Pipe;
#[cfg(target_os = "linux")]
use std::borrow::Cow;

/// Mockable APIs to interact with the system.
///
/// Each method delegates to the corresponding [`sysinfo::Disk`] method,
/// enabling dependency injection for testing.
pub trait Api {
    type Disk;
    fn get_disk_kind(disk: &Self::Disk) -> DiskKind;
    fn get_disk_name(disk: &Self::Disk) -> &OsStr;
    fn get_mount_point(disk: &Self::Disk) -> &Path;
    fn canonicalize(path: &Path) -> io::Result<PathBuf>;
}

/// Implementation of [`Api`] that interacts with the real system.
pub struct RealApi;
impl Api for RealApi {
    type Disk = Disk;

    #[inline]
    fn get_disk_kind(disk: &Self::Disk) -> DiskKind {
        disk.kind()
    }

    #[inline]
    fn get_disk_name(disk: &Self::Disk) -> &OsStr {
        disk.name()
    }

    #[inline]
    fn get_mount_point(disk: &Self::Disk) -> &Path {
        disk.mount_point()
    }

    #[inline]
    fn canonicalize(path: &Path) -> io::Result<PathBuf> {
        canonicalize(path)
    }
}

/// On Linux, the `rotational` sysfs flag defaults to `1` for virtual block devices
/// (e.g. VirtIO, Xen) because the kernel cannot determine the backing storage type.
/// This causes `sysinfo` to falsely report them as HDDs.
///
/// This function checks the block device's driver via sysfs and reclassifies
/// known virtual drivers as `Unknown` instead of `HDD`.
#[cfg(target_os = "linux")]
fn correct_hdd_detection(kind: DiskKind, disk_name: &str) -> DiskKind {
    if kind != DiskKind::HDD {
        return kind;
    }
    if let Some(block_dev) = extract_block_device_name(disk_name) {
        if is_virtual_block_device(&block_dev) {
            return DiskKind::Unknown(-1);
        }
    }
    DiskKind::HDD
}

/// On non-Linux platforms (macOS, FreeBSD), `sysinfo` currently reports
/// `DiskKind::Unknown` because there is no reliable OS API for determining
/// rotational vs solid-state. This means the `kind == DiskKind::HDD` check
/// in [`is_in_hdd`] never matches, so this function is effectively a no-op.
///
/// If `sysinfo` ever gains accurate disk-kind detection on these platforms,
/// this function should be revisited — virtual disks on macOS (e.g. virtio
/// in QEMU) or FreeBSD (e.g. virtio-blk) could face the same misclassification.
#[cfg(not(target_os = "linux"))]
fn correct_hdd_detection(kind: DiskKind, _disk_name: &str) -> DiskKind {
    kind
}

/// Resolve a device path through symlinks and then parse the block device name.
///
/// Handles `/dev/mapper/xxx` symlinks and `/dev/root` by following them via
/// `canonicalize`, then delegates to [`parse_block_device_name`] for parsing
/// and [`validate_block_device`] to verify the device exists in sysfs.
#[cfg(target_os = "linux")]
fn extract_block_device_name(device_path: &str) -> Option<Cow<'_, str>> {
    use std::fs;

    if device_path.starts_with("/dev/mapper/") || device_path.starts_with("/dev/root") {
        let real = fs::canonicalize(device_path).ok()?;
        let real_str = real.to_str()?;
        if real_str != device_path {
            return real_str
                .pipe(extract_block_device_name)
                .map(|x| x.to_string())
                .map(Cow::Owned);
        }
        return None;
    }

    let block_dev = parse_block_device_name(device_path)?;
    // validate_block_device(block_dev)
    block_dev.pipe(validate_block_device).map(Cow::Borrowed)
}

/// Parse the base block device name from a device path (pure string parsing).
///
/// This function performs no I/O; it only strips the `/dev/` prefix and
/// partition suffixes to recover the base block device name.
///
/// # Examples
///
/// - `/dev/vda1` → `Some("vda")`
/// - `/dev/sda1` → `Some("sda")`
/// - `/dev/xvda1` → `Some("xvda")`
/// - `/dev/nvme0n1p1` → `Some("nvme0n1")`
/// - `/dev/mmcblk0p1` → `Some("mmcblk0")`
/// - `vda1` (no `/dev/` prefix) → `None`
#[cfg(target_os = "linux")]
fn parse_block_device_name(device_path: &str) -> Option<&str> {
    let name = device_path.strip_prefix("/dev/")?;

    let block_dev = if name.starts_with("sd") || name.starts_with("vd") || name.starts_with("xvd") {
        // Strip trailing partition digits: "sda1" → "sda", "vda1" → "vda"
        name.trim_end_matches(|c: char| c.is_ascii_digit())
    } else if name.starts_with("nvme") || name.starts_with("mmcblk") {
        // Strip partition suffix: "nvme0n1p1" → "nvme0n1", "mmcblk0p1" → "mmcblk0"
        match name.rfind('p') {
            Some(idx)
                if idx > 0
                    && name
                        .as_bytes()
                        .get(idx + 1)
                        .is_some_and(|b| b.is_ascii_digit()) =>
            {
                &name[..idx]
            }
            _ => name,
        }
    } else {
        name
    };

    Some(block_dev)
}

/// Verify that a block device exists in sysfs.
///
/// Returns `Some(block_dev)` if `/sys/block/<block_dev>` exists, `None` otherwise.
#[cfg(target_os = "linux")]
fn validate_block_device(block_dev: &str) -> Option<&str> {
    let sysfs_path = Path::new("/sys/block").join(block_dev);
    if sysfs_path.exists() {
        Some(block_dev)
    } else {
        None
    }
}

/// Check if a block device is backed by a virtual driver.
///
/// Reads the driver symlink at `/sys/block/<dev>/device/driver` and checks
/// if it matches known virtual block device drivers.
#[cfg(target_os = "linux")]
fn is_virtual_block_device(block_dev: &str) -> bool {
    use std::fs;

    let driver_path = Path::new("/sys/block")
        .join(block_dev)
        .join("device/driver");

    let Ok(target) = fs::read_link(&driver_path) else {
        return false;
    };

    let driver_name = target
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();

    matches!(
        driver_name,
        "virtio_blk" | "xen_blkfront" | "vmw_pvscsi" | "hv_storvsc"
    )
}

/// Check if any path is in any HDD.
pub fn any_path_is_in_hdd<Api: self::Api>(paths: &[PathBuf], disks: &[Api::Disk]) -> bool {
    paths
        .iter()
        .filter_map(|file| Api::canonicalize(file).ok())
        .any(|path| path_is_in_hdd::<Api>(&path, disks))
}

/// Check if path is in any HDD.
///
/// Applies [`correct_hdd_detection`] to each disk's reported kind to work
/// around virtual block devices being falsely reported as HDDs on Linux.
fn path_is_in_hdd<Api: self::Api>(path: &Path, disks: &[Api::Disk]) -> bool {
    let Some(mount_point) = find_mount_point(path, disks.iter().map(Api::get_mount_point)) else {
        return false;
    };
    disks
        .iter()
        .filter(|disk| is_in_hdd::<Api>(disk))
        .any(|disk| Api::get_mount_point(disk) == mount_point)
}

/// Check if a disk is an HDD after applying platform-specific corrections.
fn is_in_hdd<Api: self::Api>(disk: &Api::Disk) -> bool {
    let kind = Api::get_disk_kind(disk);
    let name = Api::get_disk_name(disk).to_str().unwrap_or_default();
    correct_hdd_detection(kind, name) == DiskKind::HDD
}

#[cfg(test)]
mod test;
