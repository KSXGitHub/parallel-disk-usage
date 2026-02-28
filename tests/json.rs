#![cfg(feature = "cli")]
#![cfg(feature = "json")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use parallel_disk_usage::{
    bytes_format::BytesFormat,
    data_tree::DataTree,
    fs_tree_builder::FsTreeBuilder,
    get_size::GetApparentSize,
    hardlink::HardlinkIgnorant,
    json_data::{JsonData, JsonTree, SchemaVersion},
    reporter::{ErrorOnlyReporter, ErrorReport},
    size::Bytes,
    visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer},
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    convert::TryInto,
    io::Write,
    process::{Command, Stdio},
};

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
    let reporter = ErrorOnlyReporter::new(ErrorReport::SILENT);
    let builder = FsTreeBuilder::builder()
        .root(workspace.to_path_buf())
        .size_getter(GetApparentSize)
        .hardlinks_recorder(&HardlinkIgnorant)
        .reporter(&reporter)
        .max_depth(10)
        .build();
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

    let tree = sample_tree();
    let visualizer = Visualizer::builder()
        .data_tree(&tree)
        .bytes_format(BytesFormat::MetricUnits)
        .direction(Direction::BottomUp)
        .bar_alignment(BarAlignment::Left)
        .column_width_distribution(ColumnWidthDistribution::total(100))
        .build();
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
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
