use super::{
    Canonicalize, DiskSource, GetDiskKind, GetDiskName, GetMountPoint, any_path_is_in_hdd,
    path_is_in_hdd,
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use sysinfo::DiskKind;

#[cfg(target_os = "linux")]
use super::{PathExists, ReadLink};

/// Fake disk value read by the disk-reading capabilities.
///
/// It carries only the three fields the capabilities expose, standing in for
/// [`sysinfo::Disk`], which a test cannot construct.
struct FakeDisk {
    kind: DiskKind,
    name: &'static str,
    mount_point: &'static str,
}

impl FakeDisk {
    fn new(kind: DiskKind, name: &'static str, mount_point: &'static str) -> Self {
        Self {
            kind,
            name,
            mount_point,
        }
    }
}

/// Fake provider whose sysfs is empty.
struct EmptySysfs;

impl DiskSource for EmptySysfs {
    type Disk = FakeDisk;
}

impl GetDiskKind for EmptySysfs {
    fn get_disk_kind(disk: &Self::Disk) -> DiskKind {
        disk.kind
    }
}

impl GetDiskName for EmptySysfs {
    fn get_disk_name(disk: &Self::Disk) -> &OsStr {
        OsStr::new(disk.name)
    }
}

impl GetMountPoint for EmptySysfs {
    fn get_mount_point(disk: &Self::Disk) -> &Path {
        Path::new(disk.mount_point)
    }
}

impl Canonicalize for EmptySysfs {
    fn canonicalize(path: &Path) -> io::Result<PathBuf> {
        path.to_path_buf().pipe(Ok)
    }
}

#[cfg(target_os = "linux")]
impl PathExists for EmptySysfs {
    fn path_exists(_: &Path) -> bool {
        false
    }
}

#[cfg(target_os = "linux")]
impl ReadLink for EmptySysfs {
    fn read_link(_: &Path) -> io::Result<PathBuf> {
        Err(io::Error::new(io::ErrorKind::NotFound, "mocked"))
    }
}

#[test]
fn test_any_path_in_hdd() {
    let disks = &[
        FakeDisk::new(DiskKind::SSD, "/dev/sda", "/"),
        FakeDisk::new(DiskKind::HDD, "/dev/sdb", "/home"),
        FakeDisk::new(DiskKind::HDD, "/dev/sdc", "/mnt/hdd-data"),
        FakeDisk::new(DiskKind::SSD, "/dev/sdd", "/mnt/ssd-data"),
        FakeDisk::new(DiskKind::HDD, "/dev/sde", "/mnt/hdd-data/repo"),
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
        assert_eq!(any_path_is_in_hdd::<EmptySysfs>(&paths, disks), *in_hdd);
    }
}

#[test]
fn test_path_in_hdd() {
    let disks = &[
        FakeDisk::new(DiskKind::SSD, "/dev/sda", "/"),
        FakeDisk::new(DiskKind::HDD, "/dev/sdb", "/home"),
        FakeDisk::new(DiskKind::HDD, "/dev/sdc", "/mnt/hdd-data"),
        FakeDisk::new(DiskKind::SSD, "/dev/sdd", "/mnt/ssd-data"),
        FakeDisk::new(DiskKind::HDD, "/dev/sde", "/mnt/hdd-data/repo"),
    ];

    for (path, in_hdd) in [
        ("/etc/fstab", false),
        ("/mnt/", false),
        ("/mnt/hdd-data/repo/test", true),
        ("/mnt/hdd-data/test/test", true),
        ("/mnt/ssd-data/test/test", false),
    ] {
        println!("CASE: {path} → {in_hdd:?}");
        assert_eq!(path_is_in_hdd::<EmptySysfs>(Path::new(path), disks), in_hdd);
    }
}
