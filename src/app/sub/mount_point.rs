use std::{ffi::OsStr, path::Path};

pub(super) fn find_mount_point<'a>(
    path: &Path,
    mount_points: impl IntoIterator<Item = &'a Path>,
) -> Option<&'a Path> {
    mount_points
        .into_iter()
        .filter(|mnt| path.starts_with(&*mnt.to_string_lossy()))
        .max_by_key(|mnt| AsRef::<OsStr>::as_ref(mnt).len()) // Mount points can be nested in each other
}

#[cfg(test)]
mod tests {
    use super::find_mount_point;
    use std::path::Path;

    #[test]
    fn test_mountpoint() {
        let mount_points = [
            Path::new("/"),
            Path::new("/home"),
            Path::new("/mnt/data"),
            Path::new("/mnt/data/repo"),
            Path::new("/mnt/repo"),
        ];

        for (path, mount_point) in &[
            ("/etc/fstab", "/"),
            ("/home/user", "/home"),
            ("/mnt/data/repo/test", "/mnt/data/repo"),
            ("/mnt/data/test/test", "/mnt/data/"),
            ("/mnt/repo/test/test", "/mnt/repo/"),
        ] {
            assert_eq!(
                find_mount_point(Path::new(path), mount_points).unwrap(),
                Path::new(mount_point)
            );
        }
    }
}
