use std::path::{Path, PathBuf};

pub fn find_mountpoint(path: &Path, mount_points: &[&Path]) -> Option<PathBuf> {
    let path = path.to_string_lossy();

    mount_points
        .iter()
        .filter(|mnt| path.starts_with(&*mnt.to_string_lossy()))
        .max_by_key(|mnt| mnt.to_string_lossy().len())
        .map(|mnt| mnt.to_path_buf())
}
