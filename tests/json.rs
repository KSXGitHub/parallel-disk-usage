#![cfg(feature = "cli")]
#![cfg(feature = "json")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use parallel_disk_usage::{
    bytes_format::BytesFormat,
    data_tree::{DataTree, Reflection},
    fs_tree_builder::FsTreeBuilder,
    json_data::{JsonData, SchemaVersion},
    reporter::{ErrorOnlyReporter, ErrorReport},
    size::Bytes,
    size_getters::GET_APPARENT_SIZE,
    visualizer::{ColumnWidthDistribution, Direction, Visualizer},
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    convert::TryInto,
    io::Write,
    process::{Command, Stdio},
};

type SampleName = String;
type SampleData = Bytes;
type SampleReflection = Reflection<SampleName, SampleData>;
type SampleTree = DataTree<SampleName, SampleData>;

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
    .into_par_sorted(|left, right| left.data().cmp(&right.data()).reverse())
}

#[test]
fn json_output() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(workspace.as_path())
        .with_arg("--json-output")
        .with_arg("--quantity=len")
        .with_arg("--min-ratio=0")
        .with_arg(workspace.as_path())
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .unit_and_tree
        .pipe(TryInto::<SampleReflection>::try_into)
        .expect("extract reflection")
        .pipe(sanitize_tree_reflection);
    dbg!(&actual);
    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_APPARENT_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
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
    let json_data = JsonData {
        schema_version: SchemaVersion,
        binary_version: None,
        unit_and_tree: sample_tree().into_reflection().into(),
    };
    let json = serde_json::to_string_pretty(&json_data).expect("convert sample tree to JSON");
    eprintln!("JSON: {}\n", json);
    let workspace = Temp::new_dir().expect("create temporary directory");
    let mut child = Command::new(PDU)
        .with_current_dir(workspace.as_path())
        .with_arg("--json-input")
        .with_arg("--bytes-format=metric")
        .with_arg("--total-width=100")
        .with_arg("--max-depth=10")
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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let visualizer = Visualizer {
        data_tree: &sample_tree(),
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

    assert_eq!(actual, expected);
}

#[test]
fn json_output_json_input() {
    let workspace = SampleWorkspace::default();

    let json_output = Command::new(PDU)
        .with_current_dir(workspace.as_path())
        .with_arg("--json-output")
        .with_arg("--quantity=len")
        .with_arg(workspace.as_path())
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .spawn()
        .expect("spawn command with --json-output");
    let actual = Command::new(PDU)
        .with_current_dir(workspace.as_path())
        .with_arg("--json-input")
        .with_arg("--bytes-format=metric")
        .with_arg("--total-width=100")
        .with_arg("--max-depth=10")
        .with_stdin(
            json_output
                .stdout
                .expect("get stdout of command with --json-output")
                .into(),
        )
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .output()
        .expect("spawn command with --json-input")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{}\n", &actual);

    let expected = Command::new(PDU)
        .with_current_dir(workspace.as_path())
        .with_arg("--bytes-format=metric")
        .with_arg("--total-width=100")
        .with_arg("--max-depth=10")
        .with_arg(workspace.as_path())
        .with_stdin(Stdio::piped())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
        .output()
        .expect("spawn command for expected")
        .pipe(stdout_text);
    eprintln!("EXPECTED:\n{}\n", expected);

    assert_eq!(actual, expected);
}
