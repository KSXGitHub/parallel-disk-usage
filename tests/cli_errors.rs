#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use maplit::btreeset;
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    collections::BTreeSet,
    path::Path,
    process::{Command, Output, Stdio},
};

fn stdio(command: Command) -> Command {
    command
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
}

#[cfg(unix)]
fn fs_permission(path: impl AsRef<Path>, permission: &'static str, recursive: bool) {
    let Output { status, stderr, .. } = Command::new("chmod")
        .pipe(|cmd| if recursive { cmd.with_arg("-R") } else { cmd })
        .with_arg(permission)
        .with_arg(path.as_ref())
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::null())
        .with_stderr(Stdio::piped())
        .output()
        .expect("run chmod command");
    inspect_stderr(&stderr);
    assert!(status.success(), "chmod fails {:?}", status);
}

#[test]
fn min_ratio_1() {
    let workspace = SampleWorkspace::default();
    let Output {
        status,
        stdout,
        stderr,
    } = Command::new(PDU)
        .with_current_dir(workspace.as_path())
        .with_arg("--min-ratio=1")
        .pipe(stdio)
        .output()
        .expect("spawn command");
    let stderr = String::from_utf8(stderr).expect("parse stderr as UTF-8");
    let stderr = stderr.trim_end();
    dbg!(&status);
    eprintln!("STDERR:\n{}\n", stderr);
    assert!(!status.success());
    assert_eq!(
        stderr,
        "error: Invalid value for '--min-ratio <min-ratio>': greater than or equal to 1"
    );
    assert_eq!(&stdout, &[] as &[u8]);
}

#[test]
fn max_depth_0() {
    let workspace = SampleWorkspace::default();
    let Output {
        status,
        stdout,
        stderr,
    } = Command::new(PDU)
        .with_current_dir(workspace.as_path())
        .with_arg("--max-depth=0")
        .pipe(stdio)
        .output()
        .expect("spawn command");
    let stderr = String::from_utf8(stderr).expect("parse stderr as UTF-8");
    let stderr = stderr.trim_end();
    dbg!(&status);
    eprintln!("STDERR:\n{}\n", stderr);
    assert!(!status.success());
    assert_eq!(
        stderr,
        "error: Invalid value for '--max-depth <max-depth>': number would be zero for non-zero type"
    );
    assert_eq!(&stdout, &[] as &[u8]);
}

#[cfg(unix)]
#[test]
fn fs_errors() {
    use std::convert::TryInto;

    use parallel_disk_usage::{
        bytes_format::BytesFormat,
        data_tree::DataTree,
        fs_tree_builder::FsTreeBuilder,
        os_string_display::OsStringDisplay,
        reporter::{ErrorOnlyReporter, ErrorReport},
        size_getters::GET_APPARENT_SIZE,
        visualizer::{ColumnWidthDistribution, Direction, Visualizer},
    };

    let workspace = SampleWorkspace::default();
    fs_permission(workspace.join("empty-dir"), "-r", false);
    fs_permission(workspace.join("nested").join("0"), "-r", false);

    let Output {
        status,
        stdout,
        stderr,
    } = Command::new(PDU)
        .with_current_dir(workspace.as_path())
        .with_arg("--min-ratio=0")
        .with_arg("--total-width=100")
        .pipe(stdio)
        .output()
        .expect("spawn command");

    let stderr = String::from_utf8(stderr).expect("parse stderr as UTF-8");
    let stdout = String::from_utf8(stdout).expect("parse stdout as UTF-8");
    dbg!(&status);
    eprintln!("STDERR+STDOUT:\n{}{}\n", &stderr, &stdout);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_APPARENT_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected_stdout = format!("{}", visualizer);
    eprintln!("EXPECTED STDOUT:\n{}\n", &expected_stdout);

    fs_permission(workspace.as_path(), "+rwx", true); // to allow SampleWorkspace destructor to clean itself

    let actual_stderr_lines: BTreeSet<_> = stderr.trim_end().lines().collect();
    let expected_stderr_lines = btreeset! {
        "\r\r[error] read_dir \"./nested/0\": Permission denied (os error 13)",
        "\r\r[error] read_dir \"./empty-dir\": Permission denied (os error 13)",
    };
    assert_eq!(actual_stderr_lines, expected_stderr_lines);

    assert_eq!(stdout, expected_stdout);
}
