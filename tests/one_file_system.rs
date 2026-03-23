//! Tests for the `--one-file-system` / `-x` flag.
//!
//! ## Unit-style test
//!
//! [`same_device_on_sample_workspace`] verifies that enabling `--one-file-system` on a
//! single-device workspace produces the same tree as without it.
//!
//! ## Integration test via FUSE
//!
//! [`cross_device_excludes_mount`] uses `squashfuse` to mount a squashfs image via FUSE
//! (no root or user namespaces required) and checks that `-x` correctly excludes entries on
//! the mounted filesystem.
//!
//! The FUSE test panics when `mksquashfs`, `squashfuse`, `/dev/fuse`, or `fusermount` are
//! unavailable. It can be excluded via `RUSTFLAGS='--cfg pdu_test_skip_cross_device'`.

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
use pipe_trait::Pipe;
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
    /// The fusermount command to use for unmounting (`"fusermount3"` or `"fusermount"`).
    fusermount: &'static str,
}

/// Probes for `squashfuse`, `mksquashfs`, and FUSE infrastructure.
///
/// Verifies:
/// 1. `mksquashfs` binary exists
/// 2. `squashfuse` binary exists
/// 3. `/dev/fuse` is accessible
/// 4. `fusermount3` (or `fusermount`) binary exists
///
/// Returns `Ok(FuseTools)` with the discovered tool paths, or `Err` with a diagnostic message.
#[cfg(target_os = "linux")]
#[cfg(not(pdu_test_skip_cross_device))]
fn fuse_probe() -> Result<FuseTools, String> {
    use std::path::Path;

    which::which("mksquashfs").map_err(|error| {
        format!("`mksquashfs` not found: {error}. Install squashfs-tools for your platform.")
    })?;

    which::which("squashfuse").map_err(|error| {
        format!("`squashfuse` not found: {error}. Install squashfuse for your platform.")
    })?;

    if !Path::new("/dev/fuse").exists() {
        return Err(
            "/dev/fuse does not exist. The FUSE kernel module may not be loaded (`modprobe fuse`)."
                .to_string(),
        );
    }

    // Prefer fusermount3 (libfuse v3, actively developed) over fusermount (libfuse v2)
    let fusermount = if which::which("fusermount3").is_ok() {
        "fusermount3"
    } else if which::which("fusermount").is_ok() {
        "fusermount"
    } else {
        return Err(
            "Neither `fusermount3` nor `fusermount` found. Install FUSE for your platform."
                .to_string(),
        );
    };

    Ok(FuseTools { fusermount })
}

/// RAII guard that unmounts a FUSE mount point on drop.
#[cfg(target_os = "linux")]
#[cfg(not(pdu_test_skip_cross_device))]
struct FuseMount {
    mount_point: std::path::PathBuf,
    fusermount: &'static str,
}

#[cfg(target_os = "linux")]
#[cfg(not(pdu_test_skip_cross_device))]
impl Drop for FuseMount {
    fn drop(&mut self) {
        use command_extra::CommandExtra;
        let status = self
            .fusermount
            .pipe(std::process::Command::new)
            .with_arg("-u")
            .with_arg(&self.mount_point)
            .status();
        match status {
            Ok(status) if status.success() => {}
            Ok(status) => eprintln!("warning: {} exited with {status}", self.fusermount),
            Err(error) => eprintln!("warning: failed to run {}: {error}", self.fusermount),
        }
    }
}

/// When a subdirectory is a mount point for a different filesystem, `-x` should exclude it.
///
/// Uses `squashfuse` to mount a squashfs image via FUSE — no root privileges or
/// user namespaces required. The image is pre-built with `mksquashfs` containing the
/// test file, so the mount is read-only (which is fine since `pdu` only reads).
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

    let fuse_tools = fuse_probe().unwrap_or_else(|reason| {
        panic!(
            "error: This test requires FUSE (`mksquashfs`, `squashfuse`, `/dev/fuse`, \
             `fusermount`) but the probe failed.\n\
             reason: {reason}\n\
             hint: Install `squashfs-tools`, `squashfuse`, and FUSE for your platform, \
             or set `RUSTFLAGS='--cfg pdu_test_skip_cross_device'` to skip this test.",
        )
    });

    let pdu = env!("CARGO_BIN_EXE_pdu");
    let temp = Temp::new_dir().expect("create temp dir for cross-device test");
    let workspace = temp.join("workspace");
    let mount_point = workspace.join("mounted");
    let image_path = temp.join("squash.img");
    let staging_dir = temp.join("staging");

    fs::create_dir_all(&mount_point).expect("create workspace and mount point");
    fs::create_dir_all(&staging_dir).expect("create staging directory");

    // Write a file on the root filesystem
    let outside_content = "A".repeat(1000);
    fs::write(workspace.join("outside.txt"), &outside_content).expect("write outside.txt");

    // Create a file in the staging directory to be packed into the squashfs image
    let inside_content = "B".repeat(2000);
    fs::write(staging_dir.join("inside.txt"), &inside_content).expect("write staging/inside.txt");

    // Build a squashfs image from the staging directory
    let mksquashfs_output = Command::new("mksquashfs")
        .with_arg(&staging_dir)
        .with_arg(&image_path)
        .with_args(["-noappend", "-quiet"])
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .output()
        .expect("run mksquashfs");
    assert!(
        mksquashfs_output.status.success(),
        "mksquashfs failed: {}",
        String::from_utf8_lossy(&mksquashfs_output.stderr),
    );

    // Mount the squashfs image via squashfuse (read-only).
    // The _fuse_mount guard ensures we unmount even if assertions panic.
    let mount_output = Command::new("squashfuse")
        .with_arg(&image_path)
        .with_arg(&mount_point)
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .output()
        .expect("run squashfuse");
    assert!(
        mount_output.status.success(),
        "squashfuse mount failed: {}",
        String::from_utf8_lossy(&mount_output.stderr),
    );
    let _fuse_mount = FuseMount {
        mount_point: mount_point.clone(),
        fusermount: fuse_tools.fusermount,
    };

    // Small delay to let FUSE settle
    thread::sleep(Duration::from_millis(100));

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
}
