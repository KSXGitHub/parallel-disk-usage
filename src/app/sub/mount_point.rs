use std::{ffi::OsStr, path::Path};

pub fn find_mount_point<'a>(
    path: &Path,
    mount_points: impl IntoIterator<Item = &'a Path>,
) -> Option<&'a Path> {
    mount_points
        .into_iter()
        .filter(|mnt| path.starts_with(mnt))
        .max_by_key(|mnt| AsRef::<OsStr>::as_ref(mnt).len()) // Mount points can be nested in each other
}

#[cfg(test)]
mod tests {
    use super::find_mount_point;
    use pretty_assertions::assert_eq;
    use std::path::Path;

    #[test]
    fn test_mount_point() {
        let all_mount_points = ["/", "/home", "/mnt/data", "/mnt/data/repo", "/mnt/repo"];

        for (path, expected_mount_point) in &[
            ("/etc/fstab", "/"),
            ("/home/user", "/home"),
            ("/mnt/data/repo/test", "/mnt/data/repo"),
            ("/mnt/data/test/test", "/mnt/data/"),
            ("/mnt/repo/test/test", "/mnt/repo/"),
        ] {
            println!("CASE: {path} → {expected_mount_point}");
            let all_mount_points = all_mount_points.map(Path::new);
            assert_eq!(
                find_mount_point(Path::new(path), all_mount_points).unwrap(),
                Path::new(expected_mount_point)
            );
        }
    }
}
