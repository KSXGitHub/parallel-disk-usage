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

/// Mockable interface to [`sysinfo::Disk`] methods.
///
/// Each method delegates to a corresponding [`sysinfo::Disk`] method,
/// enabling dependency injection for testing.
pub trait DiskApi {
    type Disk;
    fn get_disk_kind(disk: &Self::Disk) -> DiskKind;
    fn get_disk_name(disk: &Self::Disk) -> &OsStr;
    fn get_mount_point(disk: &Self::Disk) -> &Path;
}

/// Mockable interface to filesystem operations.
///
/// Abstracts system calls like [`canonicalize`], [`Path::exists`], and
/// [`std::fs::read_link`] so tests can substitute an in-memory fake.
pub trait FsApi {
    fn canonicalize(path: &Path) -> io::Result<PathBuf>;
    #[cfg(target_os = "linux")]
    fn path_exists(path: &Path) -> bool;
    #[cfg(target_os = "linux")]
    fn read_link(path: &Path) -> io::Result<PathBuf>;
}

/// Implementation of [`DiskApi`] and [`FsApi`] that interacts with the real system.
pub struct RealApi;

impl DiskApi for RealApi {
    type Disk = Disk;

    fn get_disk_kind(disk: &Self::Disk) -> DiskKind {
        disk.kind()
    }

    fn get_disk_name(disk: &Self::Disk) -> &OsStr {
        disk.name()
    }

    fn get_mount_point(disk: &Self::Disk) -> &Path {
        disk.mount_point()
    }
}

impl FsApi for RealApi {
    fn canonicalize(path: &Path) -> io::Result<PathBuf> {
        canonicalize(path)
    }

    #[cfg(target_os = "linux")]
    fn path_exists(path: &Path) -> bool {
        path.exists()
    }

    #[cfg(target_os = "linux")]
    fn read_link(path: &Path) -> io::Result<PathBuf> {
        std::fs::read_link(path)
    }
}

/// On Linux, the `rotational` sysfs flag defaults to `1` for virtual block devices
/// (e.g. VirtIO, Xen) because the kernel cannot determine the backing storage type.
/// This causes `sysinfo` to falsely report them as HDDs.
///
/// This function checks the block device's driver via sysfs and reclassifies
/// known virtual drivers as `Unknown` instead of `HDD`.
#[cfg(target_os = "linux")]
fn correct_hdd_detection<Fs: FsApi>(kind: DiskKind, disk_name: &str) -> DiskKind {
    if kind != DiskKind::HDD {
        return kind;
    }
    if let Some(block_dev) = extract_block_device_name::<Fs>(disk_name) {
        if is_virtual_block_device::<Fs>(&block_dev) {
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
fn correct_hdd_detection<Fs: FsApi>(kind: DiskKind, _disk_name: &str) -> DiskKind {
    kind
}

/// Resolve a device path through symlinks and then parse the block device name.
///
/// Handles `/dev/mapper/xxx` symlinks and `/dev/root` by following them via
/// `canonicalize`, then delegates to [`parse_block_device_name`] for parsing
/// and [`validate_block_device`] to verify the device exists in sysfs.
#[cfg(target_os = "linux")]
fn extract_block_device_name<Fs: FsApi>(device_path: &str) -> Option<Cow<'_, str>> {
    if !device_path.starts_with("/dev/mapper/") && !device_path.starts_with("/dev/root") {
        let block_dev = parse_block_device_name(device_path)?;
        return block_dev
            .pipe(|name| validate_block_device::<Fs>(name))
            .map(Cow::Borrowed);
    }

    let canon_device_path = Fs::canonicalize(Path::new(device_path)).ok()?;
    let canon_device_path = canon_device_path.to_str()?;
    if canon_device_path == device_path {
        return None;
    }

    canon_device_path
        .pipe(extract_block_device_name::<Fs>)
        .map(|x| x.to_string()) // must copy-allocate because `canon_device_path` is locally owned
        .map(Cow::Owned)
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
fn validate_block_device<Fs: FsApi>(block_dev: &str) -> Option<&str> {
    "/sys/block"
        .pipe(Path::new)
        .join(block_dev)
        .pipe(|path| Fs::path_exists(&path))
        .then_some(block_dev)
}

/// Check if a block device is backed by a virtual driver.
///
/// Reads the driver symlink at `/sys/block/<dev>/device/driver` and checks
/// if it matches known virtual block device drivers.
#[cfg(target_os = "linux")]
fn is_virtual_block_device<Fs: FsApi>(block_dev: &str) -> bool {
    let driver_path = "/sys/block"
        .pipe(Path::new)
        .join(block_dev)
        .join("device/driver");

    let Ok(target) = Fs::read_link(&driver_path) else {
        return false;
    };

    let driver_name = target.file_name().and_then(OsStr::to_str);

    matches!(
        driver_name,
        Some("virtio_blk" | "xen_blkfront" | "vmw_pvscsi" | "hv_storvsc")
    )
}

/// Check if any path is in any HDD.
pub fn any_path_is_in_hdd<D: DiskApi, F: FsApi>(paths: &[PathBuf], disks: &[D::Disk]) -> bool {
    paths
        .iter()
        .filter_map(|file| F::canonicalize(file).ok())
        .any(|path| path_is_in_hdd::<D, F>(&path, disks))
}

/// Check if path is in any HDD.
///
/// Applies [`correct_hdd_detection`] to each disk's reported kind to work
/// around virtual block devices being falsely reported as HDDs on Linux.
fn path_is_in_hdd<D: DiskApi, F: FsApi>(path: &Path, disks: &[D::Disk]) -> bool {
    let Some(mount_point) = find_mount_point(path, disks.iter().map(D::get_mount_point)) else {
        return false;
    };
    disks
        .iter()
        .filter(|disk| is_in_hdd::<D, F>(disk))
        .any(|disk| D::get_mount_point(disk) == mount_point)
}

/// Check if a disk is an HDD after applying platform-specific corrections.
fn is_in_hdd<D: DiskApi, F: FsApi>(disk: &D::Disk) -> bool {
    let kind = D::get_disk_kind(disk);
    let name = D::get_disk_name(disk).to_str();
    match name {
        Some(name) => correct_hdd_detection::<F>(kind, name) == DiskKind::HDD,
        None => kind == DiskKind::HDD, // can't parse name, keep original classification
    }
}

#[cfg(test)]
mod test;
