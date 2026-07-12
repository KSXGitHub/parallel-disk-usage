use super::{
    Canonicalize, DiskSource, GetDiskKind, GetDiskName, GetMountPoint, PathExists, ReadLink,
    any_path_is_in_hdd, path_is_in_hdd,
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use sysinfo::DiskKind;

/// Declare, inside the calling test, a function-scoped `DISKS` fixture and a
/// zero-sized `FakeDisk` provider that reads it, so no state is shared between
/// tests.
macro_rules! empty_sysfs_fake {
    () => {
        static DISKS: &[(DiskKind, &str, &str)] = &[
            (DiskKind::SSD, "/dev/sda", "/"),
            (DiskKind::HDD, "/dev/sdb", "/home"),
            (DiskKind::HDD, "/dev/sdc", "/mnt/hdd-data"),
            (DiskKind::SSD, "/dev/sdd", "/mnt/ssd-data"),
            (DiskKind::HDD, "/dev/sde", "/mnt/hdd-data/repo"),
        ];

        struct FakeDisk;

        impl DiskSource for FakeDisk {
            type Disk = (DiskKind, &'static str, &'static str);
        }

        impl GetDiskKind for FakeDisk {
            fn get_disk_kind(disk: &Self::Disk) -> DiskKind {
                disk.0
            }
        }

        impl GetDiskName for FakeDisk {
            fn get_disk_name(disk: &Self::Disk) -> &OsStr {
                OsStr::new(disk.1)
            }
        }

        impl GetMountPoint for FakeDisk {
            fn get_mount_point(disk: &Self::Disk) -> &Path {
                Path::new(disk.2)
            }
        }

        impl Canonicalize for FakeDisk {
            fn canonicalize(path: &Path) -> io::Result<PathBuf> {
                path.to_path_buf().pipe(Ok)
            }
        }

        impl PathExists for FakeDisk {
            fn path_exists(_: &Path) -> bool {
                false
            }
        }

        impl ReadLink for FakeDisk {
            fn read_link(_: &Path) -> io::Result<PathBuf> {
                Err(io::Error::new(io::ErrorKind::NotFound, "mocked"))
            }
        }
    };
}

#[test]
fn test_any_path_in_hdd() {
    empty_sysfs_fake!();

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
        assert_eq!(any_path_is_in_hdd::<FakeDisk>(&paths, DISKS), *in_hdd);
    }
}

#[test]
fn test_path_in_hdd() {
    empty_sysfs_fake!();

    for (path, in_hdd) in [
        ("/etc/fstab", false),
        ("/mnt/", false),
        ("/mnt/hdd-data/repo/test", true),
        ("/mnt/hdd-data/test/test", true),
        ("/mnt/ssd-data/test/test", false),
    ] {
        println!("CASE: {path} → {in_hdd:?}");
        assert_eq!(path_is_in_hdd::<FakeDisk>(Path::new(path), DISKS), in_hdd);
    }
}
