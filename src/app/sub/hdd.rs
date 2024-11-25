use super::mount_point::find_mount_point;
use std::{
    fs,
    path::{Path, PathBuf},
};
use sysinfo::DiskKind;

pub fn detect_hdd_in_files<Disk>(
    disks: &[Disk],
    files: &[PathBuf],
    get_disk_kind: impl Fn(&Disk) -> DiskKind,
    get_mount_point: impl Fn(&Disk) -> &Path + Copy,
) -> bool {
    files
        .iter()
        .filter_map(|file| fs::canonicalize(file).ok())
        .any(|path| {
            if let Some(mount_point) = find_mount_point(&path, disks.iter().map(get_mount_point)) {
                disks.iter().any(|disk| {
                    get_disk_kind(disk) == DiskKind::HDD && get_mount_point(disk) == mount_point
                })
            } else {
                false
            }
        })
}
