//! Tests for the `--one-file-system` / `-x` flag.
//!
//! ## Unit-style test
//!
//! [`same_device_on_sample_workspace`] verifies that enabling `--one-file-system` on a
//! single-device workspace produces the same tree as without it.
//!
//! ## Integration test via FUSE
//!
//! [`cross_device_excludes_mount`] uses `fuse2fs` to mount an ext2 filesystem image via FUSE
//! (no root or user namespaces required) and checks that `-x` correctly excludes entries on
//! the mounted filesystem.
//!
//! The FUSE test panics when `fuse2fs`, `/dev/fuse`, or `fusermount` are unavailable.
//! It can be excluded via `RUSTFLAGS='--cfg pdu_test_skip_cross_device'`.

#![cfg(unix)]
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use parallel_disk_usage::{
    data_tree::DataTree,
    fs_tree_builder::FsTreeBuilder,
    get_size::GetApparentSize,
    hardlink::HardlinkIgnorant,
    os_string_display::OsStringDisplay,
    reporter::{ErrorOnlyReporter, ErrorReport},
    size::Bytes,
};
use pretty_assertions::assert_eq;

/// When all files reside on a single filesystem, `one_file_system: true` should produce
/// the same tree as `one_file_system: false`.
#[test]
fn same_device_on_sample_workspace() {
    let workspace = SampleWorkspace::default();

    let build_tree = |one_file_system: bool| -> DataTree<OsStringDisplay, Bytes> {
        FsTreeBuilder {
            root: workspace.to_path_buf(),
            size_getter: GetApparentSize,
            hardlinks_recorder: &HardlinkIgnorant,
            reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
            one_file_system,
            max_depth: 10,
        }
        .into()
    };

    let tree_without = build_tree(false)
        .into_par_sorted(|left, right| left.name().cmp(right.name()))
        .into_reflection();
    let tree_with = build_tree(true)
        .into_par_sorted(|left, right| left.name().cmp(right.name()))
        .into_reflection();

    assert_eq!(
        sanitize_tree_reflection(tree_without),
        sanitize_tree_reflection(tree_with),
        "one_file_system should not change the result when all files are on the same device",
    );
}

/// Information about the available FUSE tools, discovered by [`fuse_probe`].
#[cfg(target_os = "linux")]
#[cfg(not(pdu_test_skip_cross_device))]
struct FuseTools {
    /// The fusermount command to use for unmounting (`"fusermount"` or `"fusermount3"`).
    fusermount: &'static str,
}

/// Probes for `fuse2fs` and FUSE infrastructure.
///
/// Verifies:
/// 1. `fuse2fs` binary exists
/// 2. `/dev/fuse` is accessible
/// 3. `fusermount` (or `fusermount3`) binary exists
///
/// Returns `Ok(FuseTools)` with the discovered tool paths, or `Err` with a diagnostic message.
#[cfg(target_os = "linux")]
#[cfg(not(pdu_test_skip_cross_device))]
fn fuse_probe() -> Result<FuseTools, String> {
    use std::{path::Path, process::Command};

    // Check that fuse2fs is installed
    Command::new("fuse2fs")
        .arg("--help")
        .output()
        .map_err(|error| {
            format!(
                "`fuse2fs` not found: {error}. \
                 Install the `fuse2fs` package (or `e2fsprogs` on distros that bundle it)."
            )
        })?;

    // Check that /dev/fuse is accessible
    if !Path::new("/dev/fuse").exists() {
        return Err(
            "/dev/fuse does not exist. The FUSE kernel module may not be loaded. \
             Try `modprobe fuse`."
                .to_string(),
        );
    }

    // Check that fusermount is available (needed for unmounting)
    let has_fusermount = Command::new("fusermount").arg("-V").output().is_ok();
    let has_fusermount3 = Command::new("fusermount3").arg("-V").output().is_ok();
    let fusermount = match (has_fusermount, has_fusermount3) {
        (true, _) => "fusermount",
        (_, true) => "fusermount3",
        _ => {
            return Err(
                "Neither `fusermount` nor `fusermount3` found. Install fuse or fuse3.".to_string(),
            );
        }
    };

    Ok(FuseTools { fusermount })
}

/// When a subdirectory is a mount point for a different filesystem, `-x` should exclude it.
///
/// Uses `fuse2fs` to mount an ext2 filesystem image via FUSE — no root privileges or
/// user namespaces required.
/// Skipped when FUSE infrastructure is unavailable.
#[test]
#[cfg(target_os = "linux")]
#[cfg(not(pdu_test_skip_cross_device))]
fn cross_device_excludes_mount() {
    use command_extra::CommandExtra;
    use std::{
        fs,
        process::{Command, Stdio},
        thread,
        time::Duration,
    };

    let fuse_tools = match fuse_probe() {
        Ok(tools) => tools,
        Err(reason) => panic!(
            "error: This test requires FUSE (`fuse2fs`, `/dev/fuse`, `fusermount`) but the probe failed.\n\
             reason: {reason}\n\
             hint: Install `fuse2fs` and `fuse3` packages, or set \
             `RUSTFLAGS='--cfg pdu_test_skip_cross_device'` to skip this test.",
        ),
    };

    let pdu = env!("CARGO_BIN_EXE_pdu");
    let temp = Temp::new_dir().expect("create temp dir for cross-device test");
    let workspace = temp.join("workspace");
    let mount_point = workspace.join("mounted");
    let image_path = temp.join("ext2.img");

    fs::create_dir_all(&mount_point).expect("create workspace and mount point");

    // Write a file on the root filesystem
    let outside_content = "A".repeat(1000);
    fs::write(workspace.join("outside.txt"), &outside_content).expect("write outside.txt");

    // Create a small ext2 filesystem image (4 MiB)
    let mkfs_output = Command::new("mkfs.ext2")
        .with_args(["-F", "-q"])
        .with_arg(&image_path)
        .with_arg("4096") // 4096 × 1K blocks = 4 MiB
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .output()
        .expect("run mkfs.ext2");
    assert!(
        mkfs_output.status.success(),
        "mkfs.ext2 failed: {}",
        String::from_utf8_lossy(&mkfs_output.stderr),
    );

    // Mount the image via fuse2fs
    let mount_output = Command::new("fuse2fs")
        .with_arg(&image_path)
        .with_arg(&mount_point)
        .with_args(["-o", "rw"])
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .output()
        .expect("run fuse2fs");
    assert!(
        mount_output.status.success(),
        "fuse2fs mount failed: {}",
        String::from_utf8_lossy(&mount_output.stderr),
    );

    // Small delay to let FUSE settle
    thread::sleep(Duration::from_millis(100));

    // Write a file on the mounted (different) filesystem
    let inside_content = "B".repeat(2000);
    let write_result = fs::write(mount_point.join("inside.txt"), &inside_content);

    // Ensure we unmount even if assertions fail
    let test_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        write_result.expect("write inside.txt on mounted filesystem");

        // Run pdu WITHOUT -x — should see both files
        let without_x = Command::new(pdu)
            .with_args(["--bytes-format=plain"])
            .with_arg(&workspace)
            .with_stdout(Stdio::piped())
            .with_stderr(Stdio::piped())
            .output()
            .expect("run pdu without -x");
        let without_x_stdout = String::from_utf8_lossy(&without_x.stdout);
        let without_x_stderr = String::from_utf8_lossy(&without_x.stderr);
        if !without_x_stderr.is_empty() {
            eprintln!("pdu (no -x) STDERR:\n{without_x_stderr}");
        }
        eprintln!("pdu (no -x) STDOUT:\n{without_x_stdout}");
        assert!(
            without_x.status.success(),
            "pdu without -x failed: {without_x_stderr}",
        );
        assert!(
            without_x_stdout.contains("inside.txt"),
            "without -x should show inside.txt:\n{without_x_stdout}",
        );
        assert!(
            without_x_stdout.contains("outside.txt"),
            "without -x should show outside.txt:\n{without_x_stdout}",
        );

        // Run pdu WITH -x — should only see outside.txt
        let with_x = Command::new(pdu)
            .with_args(["--bytes-format=plain", "-x"])
            .with_arg(&workspace)
            .with_stdout(Stdio::piped())
            .with_stderr(Stdio::piped())
            .output()
            .expect("run pdu with -x");
        let with_x_stdout = String::from_utf8_lossy(&with_x.stdout);
        let with_x_stderr = String::from_utf8_lossy(&with_x.stderr);
        if !with_x_stderr.is_empty() {
            eprintln!("pdu (-x) STDERR:\n{with_x_stderr}");
        }
        eprintln!("pdu (-x) STDOUT:\n{with_x_stdout}");
        assert!(
            with_x.status.success(),
            "pdu with -x failed: {with_x_stderr}",
        );
        assert!(
            with_x_stdout.contains("outside.txt"),
            "with -x should show outside.txt:\n{with_x_stdout}",
        );
        assert!(
            !with_x_stdout.contains("inside.txt"),
            "with -x should exclude inside.txt (on different filesystem):\n{with_x_stdout}",
        );
    }));

    // Always unmount using the fusermount variant discovered by fuse_probe
    let unmount_status = Command::new(fuse_tools.fusermount)
        .with_arg("-u")
        .with_arg(&mount_point)
        .status();
    match unmount_status {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!("warning: {} exited with {status}", fuse_tools.fusermount),
        Err(error) => eprintln!("warning: failed to run {}: {error}", fuse_tools.fusermount),
    }

    if let Err(payload) = test_result {
        std::panic::resume_unwind(payload);
    }
}
