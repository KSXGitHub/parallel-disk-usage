use super::mount_point::find_mount_point;
use std::{
    fs::canonicalize,
    io,
    path::{Path, PathBuf},
};
use sysinfo::{Disk, DiskKind};

pub trait Api {
    type Disk;
    fn get_disk_kind(disk: &Self::Disk) -> DiskKind;
    fn get_mount_point(disk: &Self::Disk) -> &Path;
    fn canonicalize(path: &Path) -> io::Result<PathBuf>;
}

pub struct RealApi;
impl Api for RealApi {
    type Disk = Disk;

    fn get_disk_kind(disk: &Self::Disk) -> DiskKind {
        disk.kind()
    }

    fn get_mount_point(disk: &Self::Disk) -> &Path {
        disk.mount_point()
    }

    fn canonicalize(path: &Path) -> io::Result<PathBuf> {
        canonicalize(path)
    }
}

pub fn any_path_is_in_hdd<Api: self::Api>(paths: &[PathBuf], disks: &[Api::Disk]) -> bool {
    paths
        .iter()
        .filter_map(|file| Api::canonicalize(file).ok())
        .any(|path| path_is_in_hdd::<Api>(&path, disks))
}

fn path_is_in_hdd<Api: self::Api>(path: &Path, disks: &[Api::Disk]) -> bool {
    if let Some(mount_point) = find_mount_point(path, disks.iter().map(Api::get_mount_point)) {
        disks.iter().any(|disk| {
            Api::get_disk_kind(disk) == DiskKind::HDD && Api::get_mount_point(disk) == mount_point
        })
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{any_path_is_in_hdd, path_is_in_hdd, Api};
    use pipe_trait::Pipe;
    use pretty_assertions::assert_eq;
    use std::path::{Path, PathBuf};
    use sysinfo::DiskKind;

    struct Disk {
        kind: DiskKind,
        mount_point: &'static Path,
    }

    struct MockedApi;
    impl Api for MockedApi {
        type Disk = Disk;

        fn get_disk_kind(disk: &Self::Disk) -> DiskKind {
            disk.kind
        }

        fn get_mount_point(disk: &Self::Disk) -> &Path {
            disk.mount_point
        }

        fn canonicalize(path: &Path) -> std::io::Result<PathBuf> {
            path.to_path_buf().pipe(Ok)
        }
    }

    #[test]
    fn test_path_in_hdd() {
        let disks = &[
            Disk {
                kind: DiskKind::SSD,
                mount_point: Path::new("/"),
            },
            Disk {
                kind: DiskKind::HDD,
                mount_point: Path::new("/home"),
            },
            Disk {
                kind: DiskKind::HDD,
                mount_point: Path::new("/mnt/data"),
            },
            Disk {
                kind: DiskKind::SSD,
                mount_point: Path::new("/mnt/repo"),
            },
            Disk {
                kind: DiskKind::HDD,
                mount_point: Path::new("/mnt/data/repo"),
            },
        ];

        for (path, in_hdd) in [
            ("/etc/fstab", false),
            ("/mnt/", false),
            ("/mnt/data/repo/test", true),
            ("/mnt/data/test/test", true),
            ("/mnt/repo/test/test", false),
        ] {
            println!("CASE: {path} → {in_hdd:?}");
            assert_eq!(path_is_in_hdd::<MockedApi>(Path::new(path), disks,), in_hdd);
        }
    }

    #[test]
    fn test_any_path_in_hdd() {
        let disks = &[
            Disk {
                kind: DiskKind::SSD,
                mount_point: Path::new("/"),
            },
            Disk {
                kind: DiskKind::HDD,
                mount_point: Path::new("/home"),
            },
            Disk {
                kind: DiskKind::HDD,
                mount_point: Path::new("/mnt/data"),
            },
            Disk {
                kind: DiskKind::SSD,
                mount_point: Path::new("/mnt/repo"),
            },
            Disk {
                kind: DiskKind::HDD,
                mount_point: Path::new("/mnt/data/repo"),
            },
        ];

        for (paths, in_hdd) in [
            (
                [
                    PathBuf::from("/etc/fstab"),
                    PathBuf::from("/home/user/file"),
                ],
                true,
            ),
            (
                [
                    PathBuf::from("/mnt/data/file"),
                    PathBuf::from("/mnt/data/repo/test"),
                ],
                true,
            ),
            (
                [PathBuf::from("/usr/share"), PathBuf::from("/mnt/repo/test")],
                false,
            ),
        ] {
            println!("CASE: {paths:?} → {in_hdd:?}");
            assert_eq!(any_path_is_in_hdd::<MockedApi>(&paths, disks), in_hdd);
        }
    }
}
