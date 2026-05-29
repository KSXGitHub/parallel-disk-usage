#![cfg(feature = "cli")]
#![cfg(feature = "json")]

pub mod _utils;
pub use _utils::*;

use assert_cmp::assert_op_expr;
use command_extra::CommandExtra;
use parallel_disk_usage::bytes_format::BytesFormat;
use parallel_disk_usage::data_tree::DataTree;
use parallel_disk_usage::device::DeviceBoundary;
use parallel_disk_usage::fs_tree_builder::FsTreeBuilder;
use parallel_disk_usage::get_size::GetApparentSize;
use parallel_disk_usage::hardlink::HardlinkIgnorant;
use parallel_disk_usage::json_data::{JsonData, JsonTree, SchemaVersion};
use parallel_disk_usage::reporter::{ErrorOnlyReporter, ErrorReport};
use parallel_disk_usage::size::Bytes;
use parallel_disk_usage::visualizer::{
    BarAlignment, ColumnWidthDistribution, Direction, Visualizer,
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::convert::TryInto;
use std::io::Write;
use std::process::{Command, Stdio};

type SampleName = String;
type SampleSize = Bytes;
type SampleJsonTree = JsonTree<SampleSize>;
type SampleTree = DataTree<SampleName, SampleSize>;

fn sample_tree() -> SampleTree {
    let dir = |name: &'static str, children: Vec<SampleTree>| {
        SampleTree::dir(name.to_string(), 1024.into(), children)
    };
    let file =
        |name: &'static str, size: u64| SampleTree::file(name.to_string(), Bytes::from(size));
    dir(
        "root",
        vec![
            file("foo", 2530),
            file("bar", 52),
            dir(
                "hello",
                vec![dir("world", vec![file("hello", 45), file("world", 54)])],
            ),
            dir("empty dir", vec![]),
            dir(
                "directory with a really long name",
                vec![dir(
                    "subdirectory with a really long name",
                    vec![file("file with a really long name", 475)],
                )],
            ),
        ],
    )
    .into_par_sorted(|left, right| left.size().cmp(&right.size()).reverse())
}

/// Sample tree whose entries are deliberately stored in ascending order of size,
/// which is the opposite of the descending order produced by the default sorting.
fn ascending_sample_tree() -> SampleTree {
    let file =
        |name: &'static str, size: u64| SampleTree::file(name.to_string(), Bytes::from(size));
    SampleTree::dir(
        "root".to_string(),
        1024.into(),
        vec![file("a", 50), file("b", 500), file("c", 5000)],
    )
}

/// Feed a tree to `pdu --json-input` and return its trimmed stdout.
fn run_json_input(tree: SampleTree, extra_args: &[&str]) -> String {
    let json_tree = JsonTree {
        tree: tree.into_reflection(),
        shared: Default::default(),
    };
    let json_data = JsonData {
        schema_version: SchemaVersion,
        binary_version: None,
        body: json_tree.into(),
    };
    let json = serde_json::to_string_pretty(&json_data).expect("convert sample tree to JSON");
    let workspace = Temp::new_dir().expect("create temporary directory");
    let mut child = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--json-input")
        .with_arg("--bytes-format=metric")
        .with_arg("--total-width=100")
        .with_args(extra_args)
        .with_stdin(Stdio::piped())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .spawn()
        .expect("spawn command");
    child
        .stdin
        .as_mut()
        .expect("get stdin of child process")
        .write_all(json.as_bytes())
        .expect("write JSON string to child process's stdin");
    child
        .wait_with_output()
        .expect("wait for output of child process")
        .pipe(stdout_text)
        .trim_end()
        .to_string()
}

#[test]
fn json_output() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--json-output")
        .with_arg("--quantity=apparent-size")
        .with_arg("--min-ratio=0")
        .with_arg(&workspace)
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .body
        .pipe(TryInto::<SampleJsonTree>::try_into)
        .expect("extract reflection")
        .tree
        .pipe(sanitize_tree_reflection);
    dbg!(&actual);
    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: GetApparentSize,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        device_boundary: DeviceBoundary::Cross,
        max_depth: 10,
    };
    let expected = builder
        .pipe(DataTree::<_, Bytes>::from)
        .into_reflection()
        .par_convert_names_to_utf8()
        .expect("convert all names from raw strings to UTF-8")
        .pipe(sanitize_tree_reflection);
    dbg!(&expected);
    assert_eq!(actual, expected);
}

#[test]
fn json_input() {
    let json_tree = JsonTree {
        tree: sample_tree().into_reflection(),
        shared: Default::default(),
    };
    let json_data = JsonData {
        schema_version: SchemaVersion,
        binary_version: None,
        body: json_tree.into(),
    };
    let json = serde_json::to_string_pretty(&json_data).expect("convert sample tree to JSON");
    eprintln!("JSON: {json}\n");
    let workspace = Temp::new_dir().expect("create temporary directory");
    let mut child = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--json-input")
        .with_arg("--bytes-format=metric")
        .with_arg("--total-width=100")
        .with_arg("--max-depth=10")
        .with_arg("--min-ratio=0")
        .with_stdin(Stdio::piped())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .spawn()
        .expect("spawn command");
    child
        .stdin
        .as_mut()
        .expect("get stdin of child process")
        .write_all(json.as_bytes())
        .expect("write JSON string to child process's stdin");
    let actual = child
        .wait_with_output()
        .expect("wait for output of child process")
        .pipe(stdout_text);
    let actual = actual.trim_end();
    eprintln!("ACTUAL:\n{actual}\n");

    let visualizer = Visualizer {
        data_tree: &sample_tree(),
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn json_input_max_depth() {
    let actual = run_json_input(sample_tree(), &["--max-depth=2", "--min-ratio=0"]);
    eprintln!("ACTUAL:\n{actual}\n");
    let unlimited = run_json_input(sample_tree(), &["--max-depth=10", "--min-ratio=0"]);
    eprintln!("UNLIMITED:\n{unlimited}\n");

    // Limiting the depth must change the output.
    assert_ne!(actual, unlimited);

    // With two levels, the root's direct children appear while their deeper
    // descendants do not. `subdirectory with a really long name` lives at depth 2
    // and renders in the unlimited run, so its absence here is caused by the limit.
    assert!(actual.contains("foo"));
    assert!(!actual.contains("subdirectory with a really long name"));
    assert!(unlimited.contains("subdirectory with a really long name"));
}

#[test]
fn json_input_min_ratio() {
    let actual = run_json_input(sample_tree(), &["--max-depth=10", "--min-ratio=0.1"]);
    eprintln!("ACTUAL:\n{actual}\n");
    let unculled = run_json_input(sample_tree(), &["--max-depth=10", "--min-ratio=0"]);
    eprintln!("UNCULLED:\n{unculled}\n");

    // Culling must change the output.
    assert_ne!(actual, unculled);

    // `foo` is far above the 10% threshold and survives, while `bar` is far below
    // it and is culled. `bar` renders in the unculled run, so its absence here is
    // caused by the culling.
    assert!(actual.contains("foo"));
    assert!(!actual.contains("bar"));
    assert!(unculled.contains("bar"));
}

#[test]
fn json_input_no_sort() {
    let actual = run_json_input(ascending_sample_tree(), &["--no-sort", "--min-ratio=0"]);
    eprintln!("ACTUAL:\n{actual}\n");
    let sorted = run_json_input(ascending_sample_tree(), &["--min-ratio=0"]);
    eprintln!("SORTED:\n{sorted}\n");

    // Sorting must change the output.
    assert_ne!(actual, sorted);

    // `--no-sort` preserves the ascending input order `a, b, c`, whereas the
    // default sorts by descending size, so their relative positions flip.
    let position = |text: &str, name: &str| text.find(name).expect("entry must be present");
    assert_op_expr!(position(&actual, "a"), >, position(&actual, "c"));
    assert_op_expr!(position(&sorted, "a"), <, position(&sorted, "c"));
}

#[test]
fn json_output_json_input() {
    let workspace = SampleWorkspace::default();

    let mut json_output = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--json-output")
        .with_arg("--quantity=apparent-size")
        .with_arg(&workspace)
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .spawn()
        .expect("spawn command with --json-output");
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--json-input")
        .with_arg("--bytes-format=metric")
        .with_arg("--total-width=100")
        .with_arg("--max-depth=10")
        .with_stdin(
            json_output
                .stdout
                .take()
                .expect("get stdout of command with --json-output")
                .into(),
        )
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .output()
        .expect("spawn command with --json-input")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let expected = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--bytes-format=metric")
        .with_arg("--total-width=100")
        .with_arg("--max-depth=10")
        .with_arg("--quantity=apparent-size")
        .with_arg(&workspace)
        .with_stdin(Stdio::piped())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .output()
        .expect("spawn command for expected")
        .pipe(stdout_text);
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);

    let json_output_status = json_output
        .wait()
        .expect("wait for the command with --json-output to terminate");
    assert!(json_output_status.success());
}
