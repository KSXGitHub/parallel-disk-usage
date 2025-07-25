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
