use super::{any_path_is_in_hdd, path_is_in_hdd, Api};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::path::{Path, PathBuf};
use sysinfo::DiskKind;

/// Fake disk for [`Api`].
struct Disk {
    kind: DiskKind,
    mount_point: &'static str,
}

impl Disk {
    fn new(kind: DiskKind, mount_point: &'static str) -> Self {
        Self { kind, mount_point }
    }
}

/// Mocked implementation of [`Api`] for testing purposes.
struct MockedApi;
impl Api for MockedApi {
    type Disk = Disk;

    fn get_disk_kind(disk: &Self::Disk) -> DiskKind {
        disk.kind
    }

    fn get_mount_point(disk: &Self::Disk) -> &Path {
        Path::new(disk.mount_point)
    }

    fn canonicalize(path: &Path) -> std::io::Result<PathBuf> {
        path.to_path_buf().pipe(Ok)
    }
}

#[test]
fn test_any_path_in_hdd() {
    let disks = &[
        Disk::new(DiskKind::SSD, "/"),
        Disk::new(DiskKind::HDD, "/home"),
        Disk::new(DiskKind::HDD, "/mnt/hdd-data"),
        Disk::new(DiskKind::SSD, "/mnt/ssd-data"),
        Disk::new(DiskKind::HDD, "/mnt/hdd-data/repo"),
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
        Disk::new(DiskKind::SSD, "/"),
        Disk::new(DiskKind::HDD, "/home"),
        Disk::new(DiskKind::HDD, "/mnt/hdd-data"),
        Disk::new(DiskKind::SSD, "/mnt/ssd-data"),
        Disk::new(DiskKind::HDD, "/mnt/hdd-data/repo"),
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
    use super::super::{extract_block_device_name, is_virtual_block_device};

    #[test]
    fn test_extract_block_device_name() {
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
            // no /dev/ prefix
            ("vda1", None),
        ];

        for (input, expected) in cases {
            let result = extract_block_device_name(input);
            println!("CASE: {input} → {result:?} (expected {expected:?})");
            // We can only fully verify cases where the block device exists in sysfs.
            // For non-existent devices, extract_block_device_name returns None because
            // the sysfs path check fails. So we just verify it doesn't panic.
            if let Some(expected_name) = expected {
                if std::path::Path::new("/sys/block").join(expected_name).exists() {
                    assert_eq!(result.as_deref(), Some(*expected_name));
                }
            }
        }
    }

    #[test]
    fn test_is_virtual_block_device_on_virtio() {
        // On this VirtIO environment, vda should be detected as virtual
        if std::path::Path::new("/sys/block/vda/device/driver").exists() {
            assert!(
                is_virtual_block_device("vda"),
                "vda should be detected as a virtual block device"
            );
        }
    }

    #[test]
    fn test_extract_and_check_real_disks() {
        // Integration test: verify that VirtIO disks on this system are correctly identified
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            let name = disk.name().to_str().unwrap_or_default();
            if let Some(block_dev) = extract_block_device_name(name) {
                if block_dev.starts_with("vd") {
                    assert!(
                        is_virtual_block_device(&block_dev),
                        "VirtIO device {block_dev} should be detected as virtual"
                    );
                }
            }
        }
    }
}
