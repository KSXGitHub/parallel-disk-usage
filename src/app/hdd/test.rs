use super::{any_path_is_in_hdd, path_is_in_hdd, DiskApi, FsApi};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};
use sysinfo::DiskKind;

/// Fake disk for [`DiskApi`].
struct Disk {
    kind: DiskKind,
    name: &'static str,
    mount_point: &'static str,
}

impl Disk {
    fn new(kind: DiskKind, name: &'static str, mount_point: &'static str) -> Self {
        Self {
            kind,
            name,
            mount_point,
        }
    }
}

/// Mocked [`DiskApi`] that returns values from the fake [`Disk`] struct.
struct MockedDiskApi;

impl DiskApi for MockedDiskApi {
    type Disk = Disk;

    fn get_disk_kind(disk: &Self::Disk) -> DiskKind {
        disk.kind
    }

    fn get_disk_name(disk: &Self::Disk) -> &OsStr {
        OsStr::new(disk.name)
    }

    fn get_mount_point(disk: &Self::Disk) -> &Path {
        Path::new(disk.mount_point)
    }
}

/// Mocked [`FsApi`] with no sysfs entries.
///
/// `canonicalize` returns the path unchanged (all paths are canonical).
/// `path_exists` returns `false` and `read_link` returns `NotFound`,
/// so [`correct_hdd_detection`](super::correct_hdd_detection) is
/// effectively a no-op: disk kinds pass through unchanged.
struct EmptyFs;

impl FsApi for EmptyFs {
    fn canonicalize(path: &Path) -> io::Result<PathBuf> {
        path.to_path_buf().pipe(Ok)
    }

    #[cfg(target_os = "linux")]
    fn path_exists(_path: &Path) -> bool {
        false
    }

    #[cfg(target_os = "linux")]
    fn read_link(_path: &Path) -> io::Result<PathBuf> {
        Err(io::Error::new(io::ErrorKind::NotFound, "mocked"))
    }
}

#[test]
fn test_any_path_in_hdd() {
    let disks = &[
        Disk::new(DiskKind::SSD, "/dev/sda", "/"),
        Disk::new(DiskKind::HDD, "/dev/sdb", "/home"),
        Disk::new(DiskKind::HDD, "/dev/sdc", "/mnt/hdd-data"),
        Disk::new(DiskKind::SSD, "/dev/sdd", "/mnt/ssd-data"),
        Disk::new(DiskKind::HDD, "/dev/sde", "/mnt/hdd-data/repo"),
    ];

    let cases: &[(&[&str], bool)] = &[
        (&[], false),
        (&["/"], false),
        (&["/home"], true),
        (&["/mnt"], false),
        (&["/mnt/ssd-data"], false),
        (&["/mnt/hdd-data"], true),
        (&["/mnt/hdd-data/repo"], true),
        (&["/etc/fstab"], false),
        (&["/home/usr/file"], true),
        (&["/home/data/repo/test"], true),
        (&["/usr/share"], false),
        (&["/mnt/ssd-data/test"], false),
        (&["/etc/fstab", "/home/user/file"], true),
        (&["/mnt/hdd-data/file", "/mnt/hdd-data/repo/test"], true),
        (&["/usr/share", "/mnt/ssd-data/test"], false),
        (
            &["/etc/fstab", "/home/user", "/mnt/hdd-data", "/usr/share"],
            true,
        ),
    ];

    for (paths, in_hdd) in cases {
        let paths: Vec<_> = paths.iter().map(PathBuf::from).collect();
        println!("CASE: {paths:?} → {in_hdd:?}");
        assert_eq!(
            any_path_is_in_hdd::<MockedDiskApi, EmptyFs>(&paths, disks),
            *in_hdd,
        );
    }
}

#[test]
fn test_path_in_hdd() {
    let disks = &[
        Disk::new(DiskKind::SSD, "/dev/sda", "/"),
        Disk::new(DiskKind::HDD, "/dev/sdb", "/home"),
        Disk::new(DiskKind::HDD, "/dev/sdc", "/mnt/hdd-data"),
        Disk::new(DiskKind::SSD, "/dev/sdd", "/mnt/ssd-data"),
        Disk::new(DiskKind::HDD, "/dev/sde", "/mnt/hdd-data/repo"),
    ];

    for (path, in_hdd) in [
        ("/etc/fstab", false),
        ("/mnt/", false),
        ("/mnt/hdd-data/repo/test", true),
        ("/mnt/hdd-data/test/test", true),
        ("/mnt/ssd-data/test/test", false),
    ] {
        println!("CASE: {path} → {in_hdd:?}");
        assert_eq!(
            path_is_in_hdd::<MockedDiskApi, EmptyFs>(Path::new(path), disks),
            in_hdd,
        );
    }
}

#[cfg(target_os = "linux")]
mod linux_tests {
    use super::super::{
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

    // ── Mocked FsApi tests ─────────────────────────────────────────────

    /// VirtIO disk reported as HDD should be reclassified as Unknown(-1).
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
        static SYSFS_DRIVER_LINKS: &[(&str, &str)] =
            &[("/sys/block/vda/device/driver", "virtio_blk")];

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

    /// Xen disk reported as HDD should be reclassified as Unknown(-1).
    mod test_xen_disk_is_reclassified {
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

    /// Physical SCSI disk reported as HDD should stay HDD.
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

    /// Device mapper path should be resolved through canonicalize and
    /// then reclassified if the underlying device is virtual.
    mod test_mapper_resolves_to_virtual_disk {
        use super::{correct_hdd_detection, FsApi};
        use pretty_assertions::assert_eq;
        use std::{
            io,
            path::{Path, PathBuf},
        };
        use sysinfo::DiskKind;

        static SYSFS_BLOCK_DEVICES: &[&str] = &["/sys/block/vda"];
        static SYSFS_DRIVER_LINKS: &[(&str, &str)] =
            &[("/sys/block/vda/device/driver", "virtio_blk")];
        static SYMLINKS: &[(&str, &str)] = &[("/dev/mapper/vg0-lv0", "/dev/vda1")];

        struct Fs;
        impl FsApi for Fs {
            fn canonicalize(path: &Path) -> io::Result<PathBuf> {
                SYMLINKS
                    .iter()
                    .find(|(p, _)| path == Path::new(*p))
                    .map(|(_, target)| PathBuf::from(*target))
                    .ok_or_else(|| {
                        // Not a symlink: return the path unchanged.
                        // (io::Result requires an error, but extract_block_device_name
                        //  only calls canonicalize on /dev/mapper/ and /dev/root paths,
                        //  then recurses with the resolved path which won't match here.)
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

    // ── Real-sysfs integration tests ───────────────────────────────────

    /// Test is_virtual_block_device against real sysfs.
    #[test]
    fn test_is_virtual_block_device_with_real_sysfs() {
        // This test only asserts when the sysfs driver path actually exists,
        // so it validates the logic on systems that have the relevant devices.
        if std::path::Path::new("/sys/block/vda/device/driver").exists() {
            assert!(
                is_virtual_block_device::<RealApi>("vda"),
                "vda should be detected as a virtual block device"
            );
        }
    }

    /// Verify that non-existent devices return false without panicking.
    #[test]
    fn test_virtual_driver_names() {
        assert!(
            !is_virtual_block_device::<RealApi>("nonexistent_device_xyz"),
            "non-existent device should not be detected as virtual"
        );
    }

    /// Integration test: verify correct detection on real system disks.
    #[test]
    fn test_extract_and_check_real_disks() {
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            let name = disk.name().to_str().unwrap_or_default();
            if let Some(block_dev) = extract_block_device_name::<RealApi>(name) {
                let sysfs_path = std::path::Path::new("/sys/block").join(block_dev.as_ref());
                assert!(
                    sysfs_path.exists(),
                    "extracted block device {block_dev} should exist in sysfs"
                );
                let _ = is_virtual_block_device::<RealApi>(&block_dev);
            }
        }
    }
}
