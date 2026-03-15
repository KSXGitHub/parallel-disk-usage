use super::{parse_block_device_name, reclassify_virtual_hdd, FsApi};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    io,
    path::{Path, PathBuf},
};
use sysinfo::DiskKind;

/// Test pure parsing of block device names — no sysfs dependency.
#[test]
fn test_parse_block_device_name() {
    let cases: &[(&str, Option<&str>)] = &[
        // sd devices
        ("/dev/sda", Some("sda")),
        ("/dev/sda1", Some("sda")),
        ("/dev/sdb3", Some("sdb")),
        // virtio devices
        ("/dev/vda", Some("vda")),
        ("/dev/vda1", Some("vda")),
        ("/dev/vdb2", Some("vdb")),
        // xen devices
        ("/dev/xvda", Some("xvda")),
        ("/dev/xvda1", Some("xvda")),
        // nvme devices
        ("/dev/nvme0n1", Some("nvme0n1")),
        ("/dev/nvme0n1p1", Some("nvme0n1")),
        // mmcblk devices
        ("/dev/mmcblk0", Some("mmcblk0")),
        ("/dev/mmcblk0p1", Some("mmcblk0")),
        // no /dev/ prefix → None
        ("vda1", None),
        // unknown device type still returns the name
        ("/dev/loop0", Some("loop0")),
    ];

    for (input, expected) in cases {
        let actual = parse_block_device_name(input);
        println!("CASE: {input} → {actual:?} (expected {expected:?})");
        assert_eq!(actual, *expected);
    }
}

/// Build a mock `FsApi` whose `canonicalize` is the identity function and
/// whose `path_exists` / `read_link` are driven by static lookup tables.
///
/// Tests that need a custom `canonicalize` (e.g. symlink resolution) define
/// their own `FsApi` impl instead.
///
/// Production code only inspects the **file-name** component of the
/// `read_link` target, so the macro uses a fixed `/drivers/{driver}` prefix.
macro_rules! identity_fs_api {
    ($fs:ident, $devices:expr, $drivers:expr) => {
        struct $fs;
        impl FsApi for $fs {
            fn canonicalize(path: &Path) -> io::Result<PathBuf> {
                path.to_path_buf().pipe(Ok)
            }
            fn path_exists(path: &Path) -> bool {
                $devices.iter().any(|p| path == Path::new(*p))
            }
            fn read_link(path: &Path) -> io::Result<PathBuf> {
                $drivers
                    .iter()
                    .find(|(p, _)| path == Path::new(*p))
                    .map(|(_, driver)| PathBuf::from(format!("/drivers/{driver}")))
                    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
            }
        }
    };
}

/// VirtIO disk reported as HDD should be reclassified as `Unknown(-1)`.
#[test]
fn test_virtio_disk_is_reclassified() {
    static DEVICES: &[&str] = &["/sys/block/vda"];
    static DRIVERS: &[(&str, &str)] = &[("/sys/block/vda/device/driver", "virtio_blk")];
    identity_fs_api!(Fs, DEVICES, DRIVERS);
    assert_eq!(
        reclassify_virtual_hdd::<Fs>(DiskKind::HDD, "/dev/vda1"),
        DiskKind::Unknown(-1),
    );
}

/// Xen disk whose sysfs driver is `vbd` (the xenbus-registered name)
/// should be reclassified as `Unknown(-1)`.
#[test]
fn test_xen_vbd_disk_is_reclassified() {
    static DEVICES: &[&str] = &["/sys/block/xvda"];
    static DRIVERS: &[(&str, &str)] = &[("/sys/block/xvda/device/driver", "vbd")];
    identity_fs_api!(Fs, DEVICES, DRIVERS);
    assert_eq!(
        reclassify_virtual_hdd::<Fs>(DiskKind::HDD, "/dev/xvda1"),
        DiskKind::Unknown(-1),
    );
}

/// Xen disk whose sysfs driver is `xen_blkfront` (the underscored kernel
/// module name) should be reclassified as `Unknown(-1)`.
#[test]
fn test_xen_blkfront_underscore_disk_is_reclassified() {
    static DEVICES: &[&str] = &["/sys/block/xvda"];
    static DRIVERS: &[(&str, &str)] = &[("/sys/block/xvda/device/driver", "xen_blkfront")];
    identity_fs_api!(Fs, DEVICES, DRIVERS);
    assert_eq!(
        reclassify_virtual_hdd::<Fs>(DiskKind::HDD, "/dev/xvda1"),
        DiskKind::Unknown(-1),
    );
}

/// Xen disk whose sysfs driver is `xen-blkfront` (the hyphenated module
/// name, which may appear on some kernel versions) should also be
/// reclassified as `Unknown(-1)`.
#[test]
fn test_xen_blkfront_hyphen_disk_is_reclassified() {
    static DEVICES: &[&str] = &["/sys/block/xvda"];
    static DRIVERS: &[(&str, &str)] = &[("/sys/block/xvda/device/driver", "xen-blkfront")];
    identity_fs_api!(Fs, DEVICES, DRIVERS);
    assert_eq!(
        reclassify_virtual_hdd::<Fs>(DiskKind::HDD, "/dev/xvda1"),
        DiskKind::Unknown(-1),
    );
}

/// VMware PVSCSI disk reported as `HDD` should be reclassified as `Unknown(-1)`.
#[test]
fn test_vmware_pvscsi_disk_is_reclassified() {
    static DEVICES: &[&str] = &["/sys/block/sda"];
    static DRIVERS: &[(&str, &str)] = &[("/sys/block/sda/device/driver", "vmw_pvscsi")];
    identity_fs_api!(Fs, DEVICES, DRIVERS);
    assert_eq!(
        reclassify_virtual_hdd::<Fs>(DiskKind::HDD, "/dev/sda1"),
        DiskKind::Unknown(-1),
    );
}

/// Hyper-V storage controller disk reported as `HDD` should be reclassified as `Unknown(-1)`.
#[test]
fn test_hyperv_storvsc_disk_is_reclassified() {
    static DEVICES: &[&str] = &["/sys/block/sda"];
    static DRIVERS: &[(&str, &str)] = &[("/sys/block/sda/device/driver", "hv_storvsc")];
    identity_fs_api!(Fs, DEVICES, DRIVERS);
    assert_eq!(
        reclassify_virtual_hdd::<Fs>(DiskKind::HDD, "/dev/sda1"),
        DiskKind::Unknown(-1),
    );
}

/// Physical SCSI disk reported as `HDD` should stay `HDD`.
#[test]
fn test_physical_disk_stays_hdd() {
    static DEVICES: &[&str] = &["/sys/block/sda"];
    static DRIVERS: &[(&str, &str)] = &[("/sys/block/sda/device/driver", "sd")];
    identity_fs_api!(Fs, DEVICES, DRIVERS);
    assert_eq!(
        reclassify_virtual_hdd::<Fs>(DiskKind::HDD, "/dev/sda1"),
        DiskKind::HDD,
    );
}

/// Synthetic scenario: `/dev/mapper/vg0-lv0` canonicalizes directly to a
/// VirtIO partition (`/dev/vda1`), exercising the symlink-resolution →
/// recursive-call → reclassify path.
///
/// **Note:** On real LVM setups, `/dev/mapper/vg0-lv0` canonicalizes to
/// `/dev/dm-0`, not a partition device. See
/// `test_mapper_dm_device_is_not_corrected` for that case.
#[test]
fn test_mapper_symlink_resolves_to_virtual_partition() {
    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(path: &Path) -> io::Result<PathBuf> {
            [("/dev/mapper/vg0-lv0", "/dev/vda1")]
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, target)| PathBuf::from(*target))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
        fn path_exists(path: &Path) -> bool {
            ["/sys/block/vda"].iter().any(|p| path == Path::new(*p))
        }
        fn read_link(path: &Path) -> io::Result<PathBuf> {
            [("/sys/block/vda/device/driver", "virtio_blk")]
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, driver)| PathBuf::from(format!("/drivers/{driver}")))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
    }

    assert_eq!(
        reclassify_virtual_hdd::<Fs>(DiskKind::HDD, "/dev/mapper/vg0-lv0"),
        DiskKind::Unknown(-1),
    );
}

/// Known limitation: on real LVM setups, `/dev/mapper/vg0-lv0` canonicalizes
/// to `/dev/dm-0`. The `dm-0` device has no `/sys/block/dm-0/device/driver`
/// symlink, so virtual-disk correction silently does nothing.
///
/// See the doc comment on [`extract_block_device_name`] for details.
#[test]
fn test_mapper_dm_device_is_not_corrected() {
    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(path: &Path) -> io::Result<PathBuf> {
            [("/dev/mapper/vg0-lv0", "/dev/dm-0")]
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, target)| PathBuf::from(*target))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
        fn path_exists(_path: &Path) -> bool {
            false
        }
        fn read_link(_path: &Path) -> io::Result<PathBuf> {
            Err(io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
    }

    // dm-0 is not a recognized block device prefix, so correction
    // cannot determine the driver — HDD classification is preserved.
    assert_eq!(
        reclassify_virtual_hdd::<Fs>(DiskKind::HDD, "/dev/mapper/vg0-lv0"),
        DiskKind::HDD,
    );
}

/// SSD disk should pass through unchanged — correction is not applied.
#[test]
fn test_ssd_is_not_corrected() {
    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(_path: &Path) -> io::Result<PathBuf> {
            panic!("canonicalize should not be called for non-HDD disks");
        }
        fn path_exists(_path: &Path) -> bool {
            panic!("path_exists should not be called for non-HDD disks");
        }
        fn read_link(_path: &Path) -> io::Result<PathBuf> {
            panic!("read_link should not be called for non-HDD disks");
        }
    }

    assert_eq!(
        reclassify_virtual_hdd::<Fs>(DiskKind::SSD, "/dev/sda1"),
        DiskKind::SSD,
    );
}
