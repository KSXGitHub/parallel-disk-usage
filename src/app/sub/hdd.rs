use super::mount_point::find_mount_point;
use std::{
    fs,
    path::{Path, PathBuf},
};
use sysinfo::DiskKind;

pub fn any_path_is_in_hdd<Disk>(
    disks: &[Disk],
    paths: &[PathBuf],
    get_disk_kind: impl Fn(&Disk) -> DiskKind + Copy,
    get_mount_point: impl Fn(&Disk) -> &Path + Copy,
) -> bool {
    paths
        .iter()
        .filter_map(|file| fs::canonicalize(file).ok())
        .any(|path| path_is_in_hdd(&path, disks, get_disk_kind, get_mount_point))
}

pub fn path_is_in_hdd<Disk>(
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
