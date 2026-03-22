//! Tests for the `--one-file-system` / `-x` flag.
//!
//! ## Unit-style test
//!
//! [`same_device_on_sample_workspace`] verifies that enabling `--one-file-system` on a
//! single-device workspace produces the same tree as without it.
//!
//! ## Integration test via `unshare`
//!
//! [`cross_device_excludes_mount`] uses `unshare --user --mount --map-root-user` to create
//! a tmpfs mount inside a user namespace (no root required) and checks that `-x` correctly
//! excludes entries on the mounted filesystem.
//!
//! The `unshare` test is skipped when user namespaces are unavailable.

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

    pretty_assertions::assert_eq!(
        sanitize_tree_reflection(tree_without),
        sanitize_tree_reflection(tree_with),
        "one_file_system should not change the result when all files are on the same device",
    );
}

/// Returns `true` if `unshare --user --mount --map-root-user` is available.
fn unshare_available() -> bool {
    std::process::Command::new("unshare")
        .args(["--user", "--mount", "--map-root-user", "true"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// When a subdirectory is a mount point for a different filesystem, `-x` should exclude it.
///
/// Uses `unshare --user --mount --map-root-user` to avoid requiring root privileges.
/// Skipped when user namespaces are unavailable.
#[test]
fn cross_device_excludes_mount() {
    if !unshare_available() {
        eprintln!("skipping cross_device_excludes_mount: unshare not available");
        return;
    }

    // Build the pdu binary path
    let pdu = env!("CARGO_BIN_EXE_pdu");

    // Run pdu both with and without -x inside a user namespace that has a tmpfs mount.
    // The shell script creates:
    //   $TMPDIR/outside.txt  (on the host filesystem)
    //   $TMPDIR/mounted/     (a tmpfs mount)
    //   $TMPDIR/mounted/inside.txt
    let script = format!(
        r#"
TMPDIR=$(mktemp -d)
mkdir -p "$TMPDIR/mounted"
mount -t tmpfs tmpfs "$TMPDIR/mounted"
printf '%s' '{}' > "$TMPDIR/outside.txt"
printf '%s' '{}' > "$TMPDIR/mounted/inside.txt"
echo "=== WITHOUT_X ==="
"{pdu}" --bytes-format=plain "$TMPDIR" 2>&1
echo "=== WITH_X ==="
"{pdu}" --bytes-format=plain -x "$TMPDIR" 2>&1
umount "$TMPDIR/mounted"
rm -rf "$TMPDIR"
"#,
        "A".repeat(1000),
        "B".repeat(2000),
    );

    let output = std::process::Command::new("unshare")
        .args([
            "--user",
            "--mount",
            "--map-root-user",
            "bash",
            "-c",
            &script,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("run unshare");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("STDOUT:\n{stdout}");
    if !stderr.is_empty() {
        eprintln!("STDERR:\n{stderr}");
    }
    assert!(output.status.success(), "unshare command failed");

    let sections: Vec<&str> = stdout.split("===").collect();
    // sections: ["", " WITHOUT_X ", "\n...\n", " WITH_X ", "\n...\n"]
    assert!(sections.len() >= 5, "unexpected output format: {stdout}",);

    let without_x = sections[2].trim();
    let with_x = sections[4].trim();

    // Without -x: should contain both "inside.txt" and "outside.txt"
    assert!(
        without_x.contains("inside.txt"),
        "without -x should show inside.txt:\n{without_x}",
    );
    assert!(
        without_x.contains("outside.txt"),
        "without -x should show outside.txt:\n{without_x}",
    );

    // With -x: should contain "outside.txt" but NOT "inside.txt"
    assert!(
        with_x.contains("outside.txt"),
        "with -x should show outside.txt:\n{with_x}",
    );
    assert!(
        !with_x.contains("inside.txt"),
        "with -x should exclude inside.txt (on different filesystem):\n{with_x}",
    );
}
