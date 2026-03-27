//! Tests for the `--one-file-system` flag.
//!
//! ## Unit-style test
//!
//! [`same_device_on_sample_workspace`] verifies that enabling `--one-file-system` on a
//! single-device workspace produces the same tree as without it.
//!
//! ## Integration test via FUSE
//!
//! [`cross_device_excludes_mount`] uses `squashfuse` to mount a squashfs image via FUSE
//! (no root or user namespaces required) and checks that `--one-file-system` correctly
//! excludes entries on the mounted filesystem.
//!
//! The FUSE test panics when `mksquashfs`, `squashfuse`, `/dev/fuse`, or `fusermount` are
//! unavailable. It can be excluded via `TEST_SKIP='cross_device_excludes_mount' ./test.sh`.

#![cfg(unix)]
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use parallel_disk_usage::{
    bytes_format::BytesFormat,
    data_tree::DataTree,
    device::DeviceBoundary,
    fs_tree_builder::FsTreeBuilder,
    get_size::GetApparentSize,
    hardlink::HardlinkIgnorant,
    os_string_display::OsStringDisplay,
    reporter::{ErrorOnlyReporter, ErrorReport},
    size::Bytes,
    visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer},
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    fs::{create_dir_all, write as write_file},
    path::Path,
    process::{Command, Stdio},
    thread::sleep,
    time::Duration,
};
use which::which;

/// When all files reside on a single filesystem, [`DeviceBoundary::Stay`] should produce
/// the same tree as [`DeviceBoundary::Cross`].
#[test]
fn same_device_on_sample_workspace() {
    let workspace = SampleWorkspace::default();

    let build_tree = |device_boundary: DeviceBoundary| -> DataTree<OsStringDisplay, Bytes> {
        FsTreeBuilder {
            root: workspace.to_path_buf(),
            size_getter: GetApparentSize,
            hardlinks_recorder: &HardlinkIgnorant,
            reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
            device_boundary,
            max_depth: 10,
        }
        .into()
    };

    let cross = build_tree(DeviceBoundary::Cross)
        .into_par_sorted(|left, right| left.name().cmp(right.name()))
        .into_reflection();
    let stay = build_tree(DeviceBoundary::Stay)
        .into_par_sorted(|left, right| left.name().cmp(right.name()))
        .into_reflection();

    assert_eq!(
        sanitize_tree_reflection(cross),
        sanitize_tree_reflection(stay),
        "DeviceBoundary should not change the result when all files are on the same device",
    );
}

/// Information about the available FUSE tools, discovered by [`fuse_probe`].
struct FuseTools {
    /// The fusermount command to use for unmounting (`"fusermount3"` or `"fusermount"`).
    fusermount: &'static str,
}

/// Probes for `squashfuse`, `mksquashfs`, and FUSE infrastructure.
///
/// Verifies:
/// 1. `mksquashfs` binary exists
/// 2. `squashfuse` binary exists
/// 3. `/dev/fuse` exists
/// 4. `fusermount3` (or `fusermount`) binary exists
///
/// Returns `Ok(FuseTools)` with the discovered tool names, or `Err` with a diagnostic message.
fn fuse_probe() -> Result<FuseTools, String> {
    which("mksquashfs").map_err(|error| {
        format!("`mksquashfs` not found: {error}. Install squashfs-tools for your platform.")
    })?;

    which("squashfuse").map_err(|error| {
        format!("`squashfuse` not found: {error}. Install squashfuse for your platform.")
    })?;

    if !Path::new("/dev/fuse").exists() {
        return Err(
            "/dev/fuse does not exist. The FUSE kernel module may not be loaded (`modprobe fuse`)."
                .to_string(),
        );
    }

    let fusermount = if which("fusermount3").is_ok() {
        "fusermount3"
    } else if which("fusermount").is_ok() {
        "fusermount"
    } else {
        return Err(
            "Neither `fusermount3` nor `fusermount` found. Install FUSE for your platform."
                .to_string(),
        );
    };

    Ok(FuseTools { fusermount })
}

/// RAII guard that unmounts a FUSE mount point when dropped.
///
/// Its sole purpose is to ensure the FUSE filesystem is cleanly unmounted (via `fusermount -u`)
/// even if the test panics, preventing stale mounts from accumulating.
struct FuseMount<'a> {
    mount_point: &'a Path,
    fusermount: &'static str,
}

impl Drop for FuseMount<'_> {
    fn drop(&mut self) {
        let status = self
            .fusermount
            .pipe(Command::new)
            .with_arg("-u")
            .with_arg(self.mount_point)
            .status();
        match status {
            Ok(status) if status.success() => {}
            Ok(status) => eprintln!("warning: {} exited with {status}", self.fusermount),
            Err(error) => eprintln!("warning: failed to run {}: {error}", self.fusermount),
        }
    }
}

/// When a subdirectory is a mount point for a different filesystem,
/// `--one-file-system` should exclude it.
///
/// Uses `squashfuse` to mount a squashfs image via FUSE — no root privileges or
/// user namespaces required. The image is pre-built with `mksquashfs` containing the
/// test file, so the mount is read-only (which is fine since `pdu` only reads).
/// Panics when FUSE infrastructure is unavailable; can be excluded via
/// `TEST_SKIP='cross_device_excludes_mount' ./test.sh`.
#[test]
#[cfg_attr(not(target_os = "linux"), ignore = "this test only works on Linux")]
fn cross_device_excludes_mount() {
    let fuse_tools = fuse_probe().unwrap_or_else(|reason| {
        panic!(
            "error: This test requires FUSE (`mksquashfs`, `squashfuse`, `/dev/fuse`, \
             `fusermount`) but the probe failed.\n\
             reason: {reason}\n\
             hint: Install `squashfs-tools`, `squashfuse`, and FUSE for your platform, \
             or rerun via `TEST_SKIP='cross_device_excludes_mount' ./test.sh` to skip this test.",
        )
    });

    let temp = Temp::new_dir().expect("create temp dir for cross-device test");
    let workspace = temp.join("workspace");
    let mount_point = workspace.join("mounted");
    let image_path = temp.join("squash.img");
    let staging_dir = temp.join("staging");

    create_dir_all(&mount_point).expect("create workspace and mount point");
    create_dir_all(&staging_dir).expect("create staging directory");

    // Write a file on the root filesystem
    let outside_content = "A".repeat(1000);
    write_file(workspace.join("outside.txt"), &outside_content).expect("write outside.txt");

    // Create a file in the staging directory to be packed into the squashfs image
    let inside_content = "B".repeat(2000);
    write_file(staging_dir.join("inside.txt"), &inside_content).expect("write staging/inside.txt");

    // Build a squashfs image from the staging directory
    let mksquashfs_output = Command::new("mksquashfs")
        .with_arg(&staging_dir)
        .with_arg(&image_path)
        .with_arg("-noappend")
        .with_arg("-quiet")
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
        mount_point: &mount_point,
        fusermount: fuse_tools.fusermount,
    };

    // Wait for the FUSE mount to become readable (exponential backoff)
    let wait_ms_base = 100;
    let retries = 5;
    let poll_result = (0..retries)
        .map(|exponent| wait_ms_base << exponent)
        .map(Duration::from_millis)
        .map(sleep)
        .filter_map(|()| mount_point.read_dir().ok())
        .find_map(|mut entry| entry.next()?.ok());
    assert!(
        poll_result.is_some(),
        "FUSE mount at {mount_point:?} not ready after {retries} retries"
    );

    let build_expected_tree = |device_boundary: DeviceBoundary| -> String {
        let builder = FsTreeBuilder {
            root: workspace.clone(),
            size_getter: GetApparentSize,
            hardlinks_recorder: &HardlinkIgnorant,
            reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
            device_boundary,
            max_depth: 10,
        };
        let mut data_tree: DataTree<OsStringDisplay, Bytes> = builder.into();
        data_tree.par_cull_insignificant_data(0.01);
        data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
        let visualizer = Visualizer::<OsStringDisplay, _> {
            data_tree: &data_tree,
            bytes_format: BytesFormat::PlainNumber,
            direction: Direction::BottomUp,
            bar_alignment: BarAlignment::Left,
            column_width_distribution: ColumnWidthDistribution::total(100),
        };
        let expected = format!("{visualizer}");
        expected.trim_end().to_string()
    };

    let run_pdu = |one_file_system: bool| -> String {
        Command::new(PDU)
            .with_arg("--quantity=apparent-size")
            .with_arg("--total-width=100")
            .with_arg("--bytes-format=plain")
            .with_args(one_file_system.then_some("--one-file-system"))
            .with_arg(&workspace)
            .with_stdin(Stdio::null())
            .with_stdout(Stdio::piped())
            .with_stderr(Stdio::piped())
            .output()
            .expect("run pdu")
            .pipe(stdout_text)
    };

    // Run pdu WITHOUT --one-file-system — should see both files
    let actual = run_pdu(false);
    let expected = build_expected_tree(DeviceBoundary::Cross);
    eprintln!("WITHOUT --one-file-system:\nACTUAL:\n{actual}\n\nEXPECTED:\n{expected}\n");
    assert_eq!(actual, expected);
    assert!(
        actual.contains("inside.txt"),
        "without --one-file-system should show inside.txt:\n{actual}",
    );
    assert!(
        actual.contains("outside.txt"),
        "without --one-file-system should show outside.txt:\n{actual}",
    );

    // Run pdu WITH --one-file-system — should only see outside.txt
    let actual = run_pdu(true);
    let expected = build_expected_tree(DeviceBoundary::Stay);
    eprintln!("WITH --one-file-system:\nACTUAL:\n{actual}\n\nEXPECTED:\n{expected}\n");
    assert_eq!(actual, expected);
    assert!(
        actual.contains("outside.txt"),
        "with --one-file-system should show outside.txt:\n{actual}",
    );
    assert!(
        !actual.contains("inside.txt"),
        "with --one-file-system should exclude inside.txt (on different filesystem):\n{actual}",
    );
}
