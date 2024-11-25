use super::mount_point::find_mount_point;
use std::{fs, path::PathBuf};
use sysinfo::{Disk, DiskKind};

pub fn detect_hdd_in_files(
    disks: &[Disk],
    files: &[PathBuf],
    get_disk_kind: impl Fn(&Disk) -> DiskKind,
) -> bool {
    files
        .iter()
        .filter_map(|file| fs::canonicalize(file).ok())
        .any(|path| {
            if let Some(mount_point) =
                find_mount_point(&path, disks.iter().map(|disk| disk.mount_point()))
            {
                disks.iter().any(|disk| {
                    get_disk_kind(disk) == DiskKind::HDD && disk.mount_point() == mount_point
                })
            } else {
                false
            }
        })
}
