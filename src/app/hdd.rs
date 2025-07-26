use super::mount_point::find_mount_point;
use std::{
    fs::canonicalize,
    io,
    path::{Path, PathBuf},
};
use sysinfo::{Disk, DiskKind};

/// Mockable APIs to interact with the system.
pub trait Api {
    type Disk;
    fn get_disk_kind(disk: &Self::Disk) -> DiskKind;
    fn get_mount_point(disk: &Self::Disk) -> &Path;
    fn canonicalize(path: &Path) -> io::Result<PathBuf>;
}

/// Implementation of [`Api`] that interacts with the real system.
pub struct RealApi;
impl Api for RealApi {
    type Disk = Disk;

    #[inline]
    fn get_disk_kind(disk: &Self::Disk) -> DiskKind {
        disk.kind()
    }

    #[inline]
    fn get_mount_point(disk: &Self::Disk) -> &Path {
        disk.mount_point()
    }

    #[inline]
    fn canonicalize(path: &Path) -> io::Result<PathBuf> {
        canonicalize(path)
    }
}

/// Check if any path is in any HDD.
pub fn any_path_is_in_hdd<Api: self::Api>(paths: &[PathBuf], disks: &[Api::Disk]) -> bool {
    paths
        .iter()
        .filter_map(|file| Api::canonicalize(file).ok())
        .any(|path| path_is_in_hdd::<Api>(&path, disks))
}

/// Check if path is in any HDD.
fn path_is_in_hdd<Api: self::Api>(path: &Path, disks: &[Api::Disk]) -> bool {
    let Some(mount_point) = find_mount_point(path, disks.iter().map(Api::get_mount_point)) else {
        return false;
    };
    disks
        .iter()
        .filter(|disk| Api::get_disk_kind(disk) == DiskKind::HDD)
        .any(|disk| Api::get_mount_point(disk) == mount_point)
}

#[cfg(test)]
mod test;
