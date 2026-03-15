use super::{
    correct_hdd_detection, extract_block_device_name, is_virtual_block_device,
    parse_block_device_name, FsApi, RealApi,
};
use pretty_assertions::assert_eq;

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

/// VirtIO disk reported as HDD should be reclassified as `Unknown(-1)`.
mod test_virtio_disk_is_reclassified {
    use super::{correct_hdd_detection, FsApi};
    use pipe_trait::Pipe;
    use pretty_assertions::assert_eq;
    use std::{
        io,
        path::{Path, PathBuf},
    };
    use sysinfo::DiskKind;

    static SYSFS_BLOCK_DEVICES: &[&str] = &["/sys/block/vda"];
    static SYSFS_DRIVER_LINKS: &[(&str, &str)] = &[("/sys/block/vda/device/driver", "virtio_blk")];

    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(path: &Path) -> io::Result<PathBuf> {
            path.to_path_buf().pipe(Ok)
        }
        fn path_exists(path: &Path) -> bool {
            SYSFS_BLOCK_DEVICES.iter().any(|p| path == Path::new(*p))
        }
        fn read_link(path: &Path) -> io::Result<PathBuf> {
            SYSFS_DRIVER_LINKS
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, driver)| PathBuf::from(format!("/sys/bus/virtio/drivers/{driver}")))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            correct_hdd_detection::<Fs>(DiskKind::HDD, "/dev/vda1"),
            DiskKind::Unknown(-1),
        );
    }
}

/// Xen disk whose sysfs driver is `vbd` (the xenbus-registered name)
/// should be reclassified as `Unknown(-1)`.
mod test_xen_vbd_disk_is_reclassified {
    use super::{correct_hdd_detection, FsApi};
    use pipe_trait::Pipe;
    use pretty_assertions::assert_eq;
    use std::{
        io,
        path::{Path, PathBuf},
    };
    use sysinfo::DiskKind;

    static SYSFS_BLOCK_DEVICES: &[&str] = &["/sys/block/xvda"];
    static SYSFS_DRIVER_LINKS: &[(&str, &str)] = &[("/sys/block/xvda/device/driver", "vbd")];

    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(path: &Path) -> io::Result<PathBuf> {
            path.to_path_buf().pipe(Ok)
        }
        fn path_exists(path: &Path) -> bool {
            SYSFS_BLOCK_DEVICES.iter().any(|p| path == Path::new(*p))
        }
        fn read_link(path: &Path) -> io::Result<PathBuf> {
            SYSFS_DRIVER_LINKS
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, driver)| PathBuf::from(format!("/sys/bus/xen/drivers/{driver}")))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            correct_hdd_detection::<Fs>(DiskKind::HDD, "/dev/xvda1"),
            DiskKind::Unknown(-1),
        );
    }
}

/// Xen disk whose sysfs driver is `xen_blkfront` (the underscored kernel
/// module name) should be reclassified as `Unknown(-1)`.
mod test_xen_blkfront_underscore_disk_is_reclassified {
    use super::{correct_hdd_detection, FsApi};
    use pipe_trait::Pipe;
    use pretty_assertions::assert_eq;
    use std::{
        io,
        path::{Path, PathBuf},
    };
    use sysinfo::DiskKind;

    static SYSFS_BLOCK_DEVICES: &[&str] = &["/sys/block/xvda"];
    static SYSFS_DRIVER_LINKS: &[(&str, &str)] =
        &[("/sys/block/xvda/device/driver", "xen_blkfront")];

    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(path: &Path) -> io::Result<PathBuf> {
            path.to_path_buf().pipe(Ok)
        }
        fn path_exists(path: &Path) -> bool {
            SYSFS_BLOCK_DEVICES.iter().any(|p| path == Path::new(*p))
        }
        fn read_link(path: &Path) -> io::Result<PathBuf> {
            SYSFS_DRIVER_LINKS
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, driver)| PathBuf::from(format!("/sys/bus/xen/drivers/{driver}")))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            correct_hdd_detection::<Fs>(DiskKind::HDD, "/dev/xvda1"),
            DiskKind::Unknown(-1),
        );
    }
}

/// Xen disk whose sysfs driver is `xen-blkfront` (the hyphenated module
/// name, which may appear on some kernel versions) should also be
/// reclassified as `Unknown(-1)`.
mod test_xen_blkfront_hyphen_disk_is_reclassified {
    use super::{correct_hdd_detection, FsApi};
    use pipe_trait::Pipe;
    use pretty_assertions::assert_eq;
    use std::{
        io,
        path::{Path, PathBuf},
    };
    use sysinfo::DiskKind;

    static SYSFS_BLOCK_DEVICES: &[&str] = &["/sys/block/xvda"];
    static SYSFS_DRIVER_LINKS: &[(&str, &str)] =
        &[("/sys/block/xvda/device/driver", "xen-blkfront")];

    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(path: &Path) -> io::Result<PathBuf> {
            path.to_path_buf().pipe(Ok)
        }
        fn path_exists(path: &Path) -> bool {
            SYSFS_BLOCK_DEVICES.iter().any(|p| path == Path::new(*p))
        }
        fn read_link(path: &Path) -> io::Result<PathBuf> {
            SYSFS_DRIVER_LINKS
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, driver)| PathBuf::from(format!("/sys/bus/xen/drivers/{driver}")))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            correct_hdd_detection::<Fs>(DiskKind::HDD, "/dev/xvda1"),
            DiskKind::Unknown(-1),
        );
    }
}

/// VMware PVSCSI disk reported as `HDD` should be reclassified as `Unknown(-1)`.
mod test_vmware_pvscsi_disk_is_reclassified {
    use super::{correct_hdd_detection, FsApi};
    use pipe_trait::Pipe;
    use pretty_assertions::assert_eq;
    use std::{
        io,
        path::{Path, PathBuf},
    };
    use sysinfo::DiskKind;

    static SYSFS_BLOCK_DEVICES: &[&str] = &["/sys/block/sda"];
    static SYSFS_DRIVER_LINKS: &[(&str, &str)] = &[("/sys/block/sda/device/driver", "vmw_pvscsi")];

    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(path: &Path) -> io::Result<PathBuf> {
            path.to_path_buf().pipe(Ok)
        }
        fn path_exists(path: &Path) -> bool {
            SYSFS_BLOCK_DEVICES.iter().any(|p| path == Path::new(*p))
        }
        fn read_link(path: &Path) -> io::Result<PathBuf> {
            SYSFS_DRIVER_LINKS
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, driver)| PathBuf::from(format!("/sys/bus/pci/drivers/{driver}")))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            correct_hdd_detection::<Fs>(DiskKind::HDD, "/dev/sda1"),
            DiskKind::Unknown(-1),
        );
    }
}

/// Hyper-V storage controller disk reported as `HDD` should be reclassified as `Unknown(-1)`.
mod test_hyperv_storvsc_disk_is_reclassified {
    use super::{correct_hdd_detection, FsApi};
    use pipe_trait::Pipe;
    use pretty_assertions::assert_eq;
    use std::{
        io,
        path::{Path, PathBuf},
    };
    use sysinfo::DiskKind;

    static SYSFS_BLOCK_DEVICES: &[&str] = &["/sys/block/sda"];
    static SYSFS_DRIVER_LINKS: &[(&str, &str)] = &[("/sys/block/sda/device/driver", "hv_storvsc")];

    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(path: &Path) -> io::Result<PathBuf> {
            path.to_path_buf().pipe(Ok)
        }
        fn path_exists(path: &Path) -> bool {
            SYSFS_BLOCK_DEVICES.iter().any(|p| path == Path::new(*p))
        }
        fn read_link(path: &Path) -> io::Result<PathBuf> {
            SYSFS_DRIVER_LINKS
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, driver)| PathBuf::from(format!("/sys/bus/vmbus/drivers/{driver}")))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            correct_hdd_detection::<Fs>(DiskKind::HDD, "/dev/sda1"),
            DiskKind::Unknown(-1),
        );
    }
}

/// Physical SCSI disk reported as `HDD` should stay `HDD`.
mod test_physical_disk_stays_hdd {
    use super::{correct_hdd_detection, FsApi};
    use pipe_trait::Pipe;
    use pretty_assertions::assert_eq;
    use std::{
        io,
        path::{Path, PathBuf},
    };
    use sysinfo::DiskKind;

    static SYSFS_BLOCK_DEVICES: &[&str] = &["/sys/block/sda"];
    static SYSFS_DRIVER_LINKS: &[(&str, &str)] = &[("/sys/block/sda/device/driver", "sd")];

    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(path: &Path) -> io::Result<PathBuf> {
            path.to_path_buf().pipe(Ok)
        }
        fn path_exists(path: &Path) -> bool {
            SYSFS_BLOCK_DEVICES.iter().any(|p| path == Path::new(*p))
        }
        fn read_link(path: &Path) -> io::Result<PathBuf> {
            SYSFS_DRIVER_LINKS
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, driver)| PathBuf::from(format!("/sys/bus/scsi/drivers/{driver}")))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            correct_hdd_detection::<Fs>(DiskKind::HDD, "/dev/sda1"),
            DiskKind::HDD,
        );
    }
}

/// Synthetic scenario: `/dev/mapper/vg0-lv0` canonicalizes directly to a
/// VirtIO partition (`/dev/vda1`), exercising the symlink-resolution →
/// recursive-call → reclassify path.
///
/// **Note:** On real LVM setups, `/dev/mapper/vg0-lv0` canonicalizes to
/// `/dev/dm-0`, not a partition device. See `test_mapper_dm_device_is_not_corrected`
/// for that case.
mod test_mapper_symlink_resolves_to_virtual_partition {
    use super::{correct_hdd_detection, FsApi};
    use pretty_assertions::assert_eq;
    use std::{
        io,
        path::{Path, PathBuf},
    };
    use sysinfo::DiskKind;

    static SYSFS_BLOCK_DEVICES: &[&str] = &["/sys/block/vda"];
    static SYSFS_DRIVER_LINKS: &[(&str, &str)] = &[("/sys/block/vda/device/driver", "virtio_blk")];
    static SYMLINKS: &[(&str, &str)] = &[("/dev/mapper/vg0-lv0", "/dev/vda1")];

    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(path: &Path) -> io::Result<PathBuf> {
            SYMLINKS
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, target)| PathBuf::from(*target))
                .ok_or_else(|| {
                    // No matching symlink in the mock: return NotFound.
                    // extract_block_device_name uses .ok() on the result,
                    // so this causes the recursion to stop.
                    io::Error::new(io::ErrorKind::NotFound, "mocked")
                })
        }
        fn path_exists(path: &Path) -> bool {
            SYSFS_BLOCK_DEVICES.iter().any(|p| path == Path::new(*p))
        }
        fn read_link(path: &Path) -> io::Result<PathBuf> {
            SYSFS_DRIVER_LINKS
                .iter()
                .find(|(p, _)| path == Path::new(*p))
                .map(|(_, driver)| PathBuf::from(format!("/sys/bus/virtio/drivers/{driver}")))
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "mocked"))
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            correct_hdd_detection::<Fs>(DiskKind::HDD, "/dev/mapper/vg0-lv0"),
            DiskKind::Unknown(-1),
        );
    }
}

/// Known limitation: on real LVM setups, `/dev/mapper/vg0-lv0` canonicalizes
/// to `/dev/dm-0`. The `dm-0` device has no `/sys/block/dm-0/device/driver`
/// symlink, so virtual-disk correction silently does nothing.
///
/// See the doc comment on [`extract_block_device_name`] for details.
mod test_mapper_dm_device_is_not_corrected {
    use super::{correct_hdd_detection, FsApi};
    use pretty_assertions::assert_eq;
    use std::{
        io,
        path::{Path, PathBuf},
    };
    use sysinfo::DiskKind;

    static SYMLINKS: &[(&str, &str)] = &[("/dev/mapper/vg0-lv0", "/dev/dm-0")];

    struct Fs;
    impl FsApi for Fs {
        fn canonicalize(path: &Path) -> io::Result<PathBuf> {
            SYMLINKS
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

    #[test]
    fn test() {
        // dm-0 is not a recognized block device prefix, so correction
        // cannot determine the driver — HDD classification is preserved.
        assert_eq!(
            correct_hdd_detection::<Fs>(DiskKind::HDD, "/dev/mapper/vg0-lv0"),
            DiskKind::HDD,
        );
    }
}

/// SSD disk should pass through unchanged — correction is not applied.
mod test_ssd_is_not_corrected {
    use super::{correct_hdd_detection, FsApi};
    use pretty_assertions::assert_eq;
    use std::{
        io,
        path::{Path, PathBuf},
    };
    use sysinfo::DiskKind;

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

    #[test]
    fn test() {
        assert_eq!(
            correct_hdd_detection::<Fs>(DiskKind::SSD, "/dev/sda1"),
            DiskKind::SSD,
        );
    }
}

/// Host-dependent smoke tests.
///
/// These tests use [`RealApi`] and read from the real `/sys` filesystem.
/// They are designed to always pass regardless of the host hardware, but
/// the code paths they exercise vary by machine. They complement the
/// hermetic mocked tests above by verifying that the detection pipeline
/// works end-to-end on real devices without panicking.
mod host_dependent_smoke_tests {
    use super::{extract_block_device_name, is_virtual_block_device, RealApi};

    /// On hosts with a `/sys/block/vda` device, exercises the detection
    /// pipeline without panicking. Silently skips if `vda` does not exist.
    #[test]
    fn real_sysfs_vda_does_not_panic() {
        if std::path::Path::new("/sys/block/vda").exists() {
            let _ = is_virtual_block_device::<RealApi>("vda");
        }
    }

    /// A non-existent device name must return `false` without panicking.
    #[test]
    fn nonexistent_device_is_not_virtual() {
        assert!(
            !is_virtual_block_device::<RealApi>("nonexistent_device_xyz"),
            "non-existent device should not be detected as virtual"
        );
    }

    /// Runs the full detection pipeline on every mounted disk.
    ///
    /// Does **not** assert any specific virtual/non-virtual classification
    /// because the result depends on the host hardware. Only verifies that
    /// the pipeline completes without panicking.
    #[test]
    fn full_pipeline_does_not_panic() {
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            let name = disk.name().to_str().unwrap_or_default();
            if let Some(block_dev) = extract_block_device_name::<RealApi>(name) {
                let _is_virtual = is_virtual_block_device::<RealApi>(&block_dev);
            }
        }
    }
}
