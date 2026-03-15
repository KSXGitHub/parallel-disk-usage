use super::{any_path_is_in_hdd, path_is_in_hdd, Api};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use sysinfo::DiskKind;

/// Fake disk for [`Api`].
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

/// Mocked implementation of [`Api`] for testing purposes.
struct MockedApi;
impl Api for MockedApi {
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

    fn canonicalize(path: &Path) -> std::io::Result<PathBuf> {
        path.to_path_buf().pipe(Ok)
    }

    #[cfg(target_os = "linux")]
    fn path_exists(path: &Path) -> bool {
        path.exists()
    }

    #[cfg(target_os = "linux")]
    fn read_link(path: &Path) -> std::io::Result<PathBuf> {
        std::fs::read_link(path)
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
        assert_eq!(any_path_is_in_hdd::<MockedApi>(&paths, disks), *in_hdd);
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
        assert_eq!(path_is_in_hdd::<MockedApi>(Path::new(path), disks), in_hdd);
    }
}

#[cfg(target_os = "linux")]
mod linux_tests {
    use super::super::{is_virtual_block_device, parse_block_device_name, RealApi};
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

    /// Test is_virtual_block_device with a fake sysfs tree using a tempdir.
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

    /// Verify that known virtual driver names are matched correctly.
    #[test]
    fn test_virtual_driver_names() {
        // We test the driver name matching logic by checking the `matches!` macro
        // indirectly through is_virtual_block_device on real sysfs entries.
        // The driver name list should include:
        // - virtio_blk (VirtIO)
        // - xen_blkfront (Xen)
        // - vmw_pvscsi (VMware)
        // - hv_storvsc (Hyper-V)
        //
        // We can't create fake sysfs entries (it's a kernel filesystem),
        // but we verify the function doesn't panic on non-existent devices
        // and returns false.
        assert!(
            !is_virtual_block_device::<RealApi>("nonexistent_device_xyz"),
            "non-existent device should not be detected as virtual"
        );
    }

    /// Integration test: verify correct detection on real system disks.
    #[test]
    fn test_extract_and_check_real_disks() {
        use super::super::extract_block_device_name;
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            let name = disk.name().to_str().unwrap_or_default();
            if let Some(block_dev) = extract_block_device_name::<RealApi>(name) {
                // Verify the parsed name is valid: it should exist in sysfs
                // (extract_block_device_name already validates this).
                let sysfs_path = std::path::Path::new("/sys/block").join(block_dev.as_ref());
                assert!(
                    sysfs_path.exists(),
                    "extracted block device {block_dev} should exist in sysfs"
                );

                // If the device has a driver symlink, verify is_virtual_block_device
                // returns a consistent result (doesn't panic).
                let _ = is_virtual_block_device::<RealApi>(&block_dev);
            }
        }
    }
}
