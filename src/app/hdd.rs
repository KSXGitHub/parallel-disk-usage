use super::host::Host;
use super::mount_point::find_mount_point;
use std::ffi::OsStr;
use std::fs::canonicalize;
use std::io;
use std::path::{Path, PathBuf};
use sysinfo::{Disk, DiskKind};

#[cfg(target_os = "linux")]
use pipe_trait::Pipe;
#[cfg(target_os = "linux")]
use std::borrow::Cow;

/// The disk value that the disk-reading capabilities operate on.
///
/// The concrete disk type is exposed as an associated type so that production
/// can read a real [`sysinfo::Disk`] while a test substitutes a lightweight
/// stand-in carrying only the fields the test needs. The disk type is chosen
/// by the provider rather than by each call site.
pub trait DiskSource {
    /// The disk value that the capabilities below read from.
    type Disk;
}

/// Capability: read the [`DiskKind`] of a disk.
pub trait GetDiskKind: DiskSource {
    fn get_disk_kind(disk: &Self::Disk) -> DiskKind;
}

/// Capability: read the device name of a disk.
pub trait GetDiskName: DiskSource {
    fn get_disk_name(disk: &Self::Disk) -> &OsStr;
}

/// Capability: read the mount point of a disk.
pub trait GetMountPoint: DiskSource {
    fn get_mount_point(disk: &Self::Disk) -> &Path;
}

/// Capability: resolve a path to its canonical form, mirroring [`std::fs::canonicalize`].
pub trait Canonicalize {
    fn canonicalize(path: &Path) -> io::Result<PathBuf>;
}

/// Capability: check whether a path exists, mirroring [`Path::exists`].
#[cfg(target_os = "linux")]
pub trait PathExists {
    fn path_exists(path: &Path) -> bool;
}

/// Capability: read a symbolic link, mirroring [`std::fs::read_link`].
#[cfg(target_os = "linux")]
pub trait ReadLink {
    fn read_link(path: &Path) -> io::Result<PathBuf>;
}

/// The capabilities the HDD-detection functions require.
///
/// This is a bound alias over the individual capability traits above, not an
/// umbrella capability of its own. It declares no method, and every side
/// effect still lives in its own single-method trait. It exists only so the
/// requirement, which varies by platform, is written once rather than repeated
/// on [`is_hdd`], [`path_is_in_hdd`], and [`any_path_is_in_hdd`].
///
/// On Linux the detection additionally probes sysfs to reclassify virtual
/// block devices, so it also needs [`PathExists`] and [`ReadLink`]. On other
/// platforms the reclassification is a no-op, so those capabilities are neither
/// required nor defined.
#[cfg(target_os = "linux")]
pub trait HddDetection:
    GetDiskKind + GetDiskName + GetMountPoint + Canonicalize + PathExists + ReadLink
{
}

#[cfg(target_os = "linux")]
impl<Sys> HddDetection for Sys where
    Sys: GetDiskKind + GetDiskName + GetMountPoint + Canonicalize + PathExists + ReadLink
{
}

/// The capabilities the HDD-detection functions require.
///
/// See the Linux definition of this trait for the full explanation. On this
/// platform the virtual-disk reclassification is a no-op, so only the
/// disk-reading capabilities and [`Canonicalize`] are needed.
#[cfg(not(target_os = "linux"))]
pub trait HddDetection: GetDiskKind + GetDiskName + GetMountPoint + Canonicalize {}

#[cfg(not(target_os = "linux"))]
impl<Sys> HddDetection for Sys where Sys: GetDiskKind + GetDiskName + GetMountPoint + Canonicalize {}

impl DiskSource for Host {
    type Disk = Disk;
}

impl GetDiskKind for Host {
    #[inline]
    fn get_disk_kind(disk: &Self::Disk) -> DiskKind {
        disk.kind()
    }
}

impl GetDiskName for Host {
    #[inline]
    fn get_disk_name(disk: &Self::Disk) -> &OsStr {
        disk.name()
    }
}

impl GetMountPoint for Host {
    #[inline]
    fn get_mount_point(disk: &Self::Disk) -> &Path {
        disk.mount_point()
    }
}

impl Canonicalize for Host {
    #[inline]
    fn canonicalize(path: &Path) -> io::Result<PathBuf> {
        canonicalize(path)
    }
}

#[cfg(target_os = "linux")]
impl PathExists for Host {
    #[inline]
    fn path_exists(path: &Path) -> bool {
        path.exists()
    }
}

#[cfg(target_os = "linux")]
impl ReadLink for Host {
    #[inline]
    fn read_link(path: &Path) -> io::Result<PathBuf> {
        std::fs::read_link(path)
    }
}

/// Sentinel value used to reclassify virtual block devices that were
/// falsely reported as `DiskKind::HDD` by `sysinfo`.
#[cfg(target_os = "linux")]
const VIRTUAL_DISK_KIND: DiskKind = DiskKind::Unknown(-1);

/// On Linux, the `rotational` sysfs flag defaults to `1` for virtual block devices
/// (e.g. VirtIO, Xen) because the kernel cannot determine the backing storage type.
/// This causes `sysinfo` to falsely report them as HDDs.
///
/// This function checks the block device's driver via sysfs and reclassifies
/// known virtual drivers as `Unknown` instead of `HDD`.
#[cfg(target_os = "linux")]
fn reclassify_virtual_hdd<Sys>(kind: DiskKind, disk_name: &str) -> DiskKind
where
    Sys: Canonicalize + PathExists + ReadLink,
{
    if kind != DiskKind::HDD {
        return kind;
    }
    if let Some(block_dev) = extract_block_device_name::<Sys>(disk_name)
        && is_virtual_block_device::<Sys>(&block_dev)
    {
        return VIRTUAL_DISK_KIND;
    }
    DiskKind::HDD
}

/// On non-Linux platforms (macOS, FreeBSD), `sysinfo` currently reports
/// `DiskKind::Unknown` because there is no reliable OS API for determining
/// rotational vs solid-state. This means the `kind == DiskKind::HDD` check
/// in [`is_hdd`] never matches, so this function is effectively a no-op.
///
/// If `sysinfo` ever gains accurate disk-kind detection on these platforms,
/// this function should be revisited. Virtual disks on macOS (e.g. virtio in
/// QEMU) or FreeBSD (e.g. virtio-blk) could then face the same misclassification.
#[cfg(not(target_os = "linux"))]
fn reclassify_virtual_hdd<Sys>(kind: DiskKind, _: &str) -> DiskKind
where
    Sys: Canonicalize,
{
    kind
}

/// Resolve a device path through symlinks and then parse the block device name.
///
/// Handles `/dev/mapper/xxx` symlinks and `/dev/root` by following them via
/// `canonicalize`, then delegates to [`parse_block_device_name`] for parsing
/// and [`validate_block_device`] to verify the device exists in sysfs.
///
/// **Known limitation:** LVM / device-mapper
///
/// On real LVM setups, `/dev/mapper/vg0-lv0` canonicalizes to `/dev/dm-0`
/// (a device-mapper device), not to the underlying physical device like
/// `/dev/vda1`. The `dm-0` device has no `/sys/block/dm-0/device/driver`
/// symlink, so [`is_virtual_block_device`] cannot determine its driver and
/// returns `false`. This means virtual-disk correction silently does nothing
/// for LVM volumes, even when the backing device is VirtIO.
///
/// Fixing this would require walking `/sys/block/dm-*/slaves/` to discover
/// the real backing device(s). That introduces three problems:
///
/// 1. A `read_dir` capability would be needed, expanding the provider and
///    every test fake.
/// 2. The slave chain can be recursive (`dm` on `dm`, e.g. LUKS on LVM),
///    requiring unbounded traversal.
/// 3. A `dm` device can have multiple slaves (stripes, mirrors). A policy
///    decision is needed: is the device virtual only when *all* slaves are
///    virtual, or when *any* is? Neither answer is obviously correct.
///
/// Given the complexity and the relative importance of the auto HDD detection feature,
/// we have chosen to ignore it.
#[cfg(target_os = "linux")]
fn extract_block_device_name<Sys>(device_path: &str) -> Option<Cow<'_, str>>
where
    Sys: Canonicalize + PathExists,
{
    if !device_path.starts_with("/dev/mapper/") && !device_path.starts_with("/dev/root") {
        let block_dev = parse_block_device_name(device_path)?;
        return block_dev
            .pipe(validate_block_device::<Sys>)
            .map(Cow::Borrowed);
    }

    let canon_device_path = Sys::canonicalize(Path::new(device_path)).ok()?;
    let canon_device_path = canon_device_path.to_str()?;
    if canon_device_path == device_path {
        return None;
    }

    // Safe to recurse: `canonicalize` resolves all symlinks, so the
    // canonical path will not start with `/dev/mapper/` or `/dev/root`.
    canon_device_path
        .pipe(extract_block_device_name::<Sys>)
        .map(Cow::into_owned) // must copy-allocate because `canon_device_path` is locally owned
        .map(Cow::Owned)
}

/// Parse the base block device name from a device path (pure string parsing).
///
/// This function performs no I/O; it only strips the `/dev/` prefix and
/// partition suffixes to recover the base block device name.
///
/// **Examples:**
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
        match name.rsplit_once('p') {
            Some((base, suffix))
                if !base.is_empty()
                    && !suffix.is_empty()
                    && suffix.bytes().all(|b| b.is_ascii_digit()) =>
            {
                base
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
fn validate_block_device<Sys>(block_dev: &str) -> Option<&str>
where
    Sys: PathExists,
{
    "/sys/block"
        .pipe(Path::new)
        .join(block_dev)
        .pipe_as_ref(Sys::path_exists)
        .then_some(block_dev)
}

/// Check if a block device is backed by a virtual driver.
///
/// Reads the driver symlink at `/sys/block/<dev>/device/driver` and checks
/// if it matches known virtual block device drivers.
#[cfg(target_os = "linux")]
fn is_virtual_block_device<Sys>(block_dev: &str) -> bool
where
    Sys: ReadLink,
{
    let driver_path = "/sys/block"
        .pipe(Path::new)
        .join(block_dev)
        .join("device/driver");

    let Ok(target) = Sys::read_link(&driver_path) else {
        return false;
    };

    let driver_name = target.file_name().and_then(OsStr::to_str);

    matches!(
        driver_name,
        Some(
            "virtio_blk"
                | "virtio-blk"
                | "xen_blkfront"
                | "xen-blkfront"
                | "vbd"
                | "vmw_pvscsi"
                | "hv_storvsc",
        ),
    )
}

/// Check if any path is in any HDD.
pub fn any_path_is_in_hdd<Sys>(paths: &[PathBuf], disks: &[Sys::Disk]) -> bool
where
    Sys: HddDetection,
{
    paths
        .iter()
        .filter_map(|file| Sys::canonicalize(file).ok())
        .any(|path| path_is_in_hdd::<Sys>(&path, disks))
}

/// Check if path is in any HDD.
///
/// Applies [`reclassify_virtual_hdd`] to each disk's reported kind to work
/// around virtual block devices being falsely reported as HDDs on Linux.
fn path_is_in_hdd<Sys>(path: &Path, disks: &[Sys::Disk]) -> bool
where
    Sys: HddDetection,
{
    let mount_point = find_mount_point(path, disks.iter().map(Sys::get_mount_point));
    let Some(mount_point) = mount_point else {
        return false;
    };
    disks
        .iter()
        .filter(|disk| Sys::get_mount_point(disk) == mount_point)
        .any(|disk| is_hdd::<Sys>(disk))
}

/// Check if a disk is an HDD after applying platform-specific corrections.
fn is_hdd<Sys>(disk: &Sys::Disk) -> bool
where
    Sys: HddDetection,
{
    let kind = Sys::get_disk_kind(disk);
    let name = Sys::get_disk_name(disk).to_str();
    match name {
        Some(name) => reclassify_virtual_hdd::<Sys>(kind, name) == DiskKind::HDD,
        None => kind == DiskKind::HDD, // can't parse name, keep original classification
    }
}

#[cfg(test)]
mod test;

#[cfg(target_os = "linux")]
#[cfg(test)]
mod test_linux;

#[cfg(target_os = "linux")]
#[cfg(test)]
mod test_linux_smoke;
