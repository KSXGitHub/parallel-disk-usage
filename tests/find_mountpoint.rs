use parallel_disk_usage::mount_point::find_mountpoint;
use std::path::Path;

#[test]
fn test_mountpoint() {
    let mount_points = &[
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
            find_mountpoint(Path::new(path), mount_points).unwrap(),
            Path::new(mount_point)
        );
    }
}
