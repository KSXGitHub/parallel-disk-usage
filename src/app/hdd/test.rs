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
