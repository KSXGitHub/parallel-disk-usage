use super::mount_point::find_mount_point;
use std::{
    io,
    path::{Path, PathBuf},
};
use sysinfo::DiskKind;

pub fn any_path_is_in_hdd<Disk>(
    paths: &[PathBuf],
    disks: &[Disk],
    get_disk_kind: impl Fn(&Disk) -> DiskKind + Copy,
    get_mount_point: impl Fn(&Disk) -> &Path + Copy,
    canonicalize: impl Fn(&Path) -> io::Result<PathBuf>,
) -> bool {
    paths
        .iter()
        .filter_map(|file| canonicalize(file).ok())
        .any(|path| path_is_in_hdd(&path, disks, get_disk_kind, get_mount_point))
}

fn path_is_in_hdd<Disk>(
    path: &Path,
    disks: &[Disk],
    get_disk_kind: impl Fn(&Disk) -> DiskKind,
    get_mount_point: impl Fn(&Disk) -> &Path + Copy,
) -> bool {
    if let Some(mount_point) = find_mount_point(path, disks.iter().map(get_mount_point)) {
        disks.iter().any(|disk| {
            get_disk_kind(disk) == DiskKind::HDD && get_mount_point(disk) == mount_point
        })
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{any_path_is_in_hdd, path_is_in_hdd};
    use pretty_assertions::assert_eq;
    use std::path::{self, Path, PathBuf};
    use sysinfo::DiskKind;

    struct Disk {
        kind: DiskKind,
        mount_point: &'static Path,
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
            assert_eq!(
                path_is_in_hdd(
                    Path::new(path),
                    disks,
                    |disk| disk.kind,
                    |disk| &disk.mount_point
                ),
                in_hdd
            );
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
            assert_eq!(
                any_path_is_in_hdd(
                    &paths,
                    disks,
                    |disk| disk.kind,
                    |disk| &disk.mount_point,
                    |path| Ok(path.to_path_buf()),
                ),
                in_hdd
            );
        }
    }
}
