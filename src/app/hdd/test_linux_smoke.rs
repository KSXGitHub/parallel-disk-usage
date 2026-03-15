use super::{extract_block_device_name, is_virtual_block_device, RealFs};

/// On hosts with a `/sys/block/vda` device, exercises the detection
/// pipeline without panicking. Silently skips if `vda` does not exist.
#[test]
fn real_sysfs_vda_does_not_panic() {
    if std::path::Path::new("/sys/block/vda").exists() {
        let _ = is_virtual_block_device::<RealFs>("vda");
    }
}

/// A non-existent device name must return `false` without panicking.
#[test]
fn nonexistent_device_is_not_virtual() {
    assert!(
        !is_virtual_block_device::<RealFs>("nonexistent_device_xyz"),
        "non-existent device should not be detected as virtual"
    );
}

/// Runs the full detection pipeline on every mounted disk.
///
/// Does **not** assert any specific virtual/non-virtual classification
/// because the result depends on the host hardware. Only verifies that
/// the pipeline completes without panicking.
#[test]
fn full_pipeline_does_not_panic() {
    use sysinfo::Disks;
    let disks = Disks::new_with_refreshed_list();
    for disk in disks.list() {
        let name = disk.name().to_str().unwrap_or_default();
        if let Some(block_dev) = extract_block_device_name::<RealFs>(name) {
            let _ = is_virtual_block_device::<RealFs>(&block_dev);
        }
    }
}
