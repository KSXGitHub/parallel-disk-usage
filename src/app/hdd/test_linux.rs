use super::{parse_block_device_name, reclassify_virtual_hdd, FsApi, VIRTUAL_DISK_KIND};
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

/// Generate a test that builds a mock `FsApi` with identity `canonicalize`,
/// then asserts that `reclassify_virtual_hdd` maps `DiskKind::HDD` to the
/// expected `DiskKind`.
///
/// The sysfs paths (`/sys/block/{block}` and
/// `/sys/block/{block}/device/driver`) are derived from `block_device`,
/// so callers only supply the four varying pieces: block device name, kernel
/// driver name, disk name, and expected `DiskKind`.
macro_rules! identity_reclassify_test_case {
    (
        $(#[$attr:meta])*
        $name:ident where
            block_device = $block:literal,
            driver = $driver:literal,
            disk_name = $disk_name:literal,
            expected = $expected:expr,
    ) => {
        $(#[$attr])*
        #[test]
        fn $name() {
            static DEVICES: &[&str] = &[concat!("/sys/block/", $block)];
            static DRIVERS: &[(&str, &str)] =
                &[(concat!("/sys/block/", $block, "/device/driver"), $driver)];

            struct Fs;
            impl FsApi for Fs {
                fn canonicalize(path: &Path) -> io::Result<PathBuf> {
                    path.to_path_buf().pipe(Ok)
                }
                fn path_exists(path: &Path) -> bool {
                    DEVICES.iter().any(|p| path == Path::new(*p))
                }
                fn read_link(path: &Path) -> io::Result<PathBuf> {
                    DRIVERS
                        .iter()
                        .find(|(p, _)| path == Path::new(*p))
                        .map(|(_, driver)| PathBuf::from(format!("/drivers/{driver}")))
                        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
                }
            }

            assert_eq!(
                reclassify_virtual_hdd::<Fs>(DiskKind::HDD, $disk_name),
                $expected,
            );
        }
    };
}

identity_reclassify_test_case! {
    /// VirtIO disk reported as HDD should be reclassified as [`VIRTUAL_DISK_KIND`].
    test_virtio_disk_is_reclassified where
        block_device = "vda",
        driver = "virtio_blk",
        disk_name = "/dev/vda1",
        expected = VIRTUAL_DISK_KIND,
}

identity_reclassify_test_case! {
    /// VirtIO disk whose sysfs driver is `virtio-blk` (the hyphenated
    /// variant) should also be reclassified as [`VIRTUAL_DISK_KIND`].
    test_virtio_blk_hyphen_disk_is_reclassified where
        block_device = "vda",
        driver = "virtio-blk",
        disk_name = "/dev/vda1",
        expected = VIRTUAL_DISK_KIND,
}

identity_reclassify_test_case! {
    /// Xen disk whose sysfs driver is `vbd` (the xenbus-registered name)
    /// should be reclassified as [`VIRTUAL_DISK_KIND`].
    test_xen_vbd_disk_is_reclassified where
        block_device = "xvda",
        driver = "vbd",
        disk_name = "/dev/xvda1",
        expected = VIRTUAL_DISK_KIND,
}

identity_reclassify_test_case! {
    /// Xen disk whose sysfs driver is `xen_blkfront` (the underscored kernel
    /// module name) should be reclassified as [`VIRTUAL_DISK_KIND`].
    test_xen_blkfront_underscore_disk_is_reclassified where
        block_device = "xvda",
        driver = "xen_blkfront",
        disk_name = "/dev/xvda1",
        expected = VIRTUAL_DISK_KIND,
}

identity_reclassify_test_case! {
    /// Xen disk whose sysfs driver is `xen-blkfront` (the hyphenated module
    /// name, which may appear on some kernel versions) should also be
    /// reclassified as [`VIRTUAL_DISK_KIND`].
    test_xen_blkfront_hyphen_disk_is_reclassified where
        block_device = "xvda",
        driver = "xen-blkfront",
        disk_name = "/dev/xvda1",
        expected = VIRTUAL_DISK_KIND,
}

identity_reclassify_test_case! {
    /// VMware PVSCSI disk reported as `HDD` should be reclassified as [`VIRTUAL_DISK_KIND`].
    test_vmware_pvscsi_disk_is_reclassified where
        block_device = "sda",
        driver = "vmw_pvscsi",
        disk_name = "/dev/sda1",
        expected = VIRTUAL_DISK_KIND,
}

identity_reclassify_test_case! {
    /// Hyper-V storage controller disk reported as `HDD` should be reclassified as [`VIRTUAL_DISK_KIND`].
    test_hyperv_storvsc_disk_is_reclassified where
        block_device = "sda",
        driver = "hv_storvsc",
        disk_name = "/dev/sda1",
        expected = VIRTUAL_DISK_KIND,
}

identity_reclassify_test_case! {
    /// Physical SCSI disk reported as `HDD` should stay `HDD`.
    test_physical_disk_stays_hdd where
        block_device = "sda",
        driver = "sd",
        disk_name = "/dev/sda1",
        expected = DiskKind::HDD,
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
        VIRTUAL_DISK_KIND,
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
        fn path_exists(path: &Path) -> bool {
            path == Path::new("/sys/block/dm-0")
        }
        fn read_link(_: &Path) -> io::Result<PathBuf> {
            Err(io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
    }

    // dm-0 is recognized but has no /sys/block/dm-0/device/driver
    // symlink, so driver detection fails — HDD classification is preserved.
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
        fn canonicalize(_: &Path) -> io::Result<PathBuf> {
            panic!("canonicalize should not be called for non-HDD disks");
        }
        fn path_exists(_: &Path) -> bool {
            panic!("path_exists should not be called for non-HDD disks");
        }
        fn read_link(_: &Path) -> io::Result<PathBuf> {
            panic!("read_link should not be called for non-HDD disks");
        }
    }

    assert_eq!(
        reclassify_virtual_hdd::<Fs>(DiskKind::SSD, "/dev/sda1"),
        DiskKind::SSD,
    );
}
