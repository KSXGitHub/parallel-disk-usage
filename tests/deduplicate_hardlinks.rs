#![cfg(unix)] // This feature is not available in Windows
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use parallel_disk_usage::{
    json_data::{JsonData, UnitAndTree},
    size::Bytes,
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    ops::{Add, Mul},
    process::{Command, Stdio},
};

fn stdio(command: Command) -> Command {
    command
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
}

#[test]
fn deduplicate_multiple_hardlinks_to_a_single_file() {
    let workspace = SampleWorkspace::multiple_hardlinks_to_a_single_file();

    let json = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("--json-output")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData");

    let actual_size = match &json.unit_and_tree {
        UnitAndTree::Blocks(_) => panic!("expecting Bytes, but got {:?}", &json.unit_and_tree),
        UnitAndTree::Bytes(tree) => tree.size,
    };

    let expected_size = workspace
        .join("file.txt")
        .pipe_as_ref(read_apparent_size)
        .mul(2) // TODO: fix the algorithm and remove this line
        .add(read_apparent_size(&workspace))
        .pipe(Bytes::new);

    assert_eq!(actual_size, expected_size);
}

#[test]
fn do_not_deduplicate_multiple_hardlinks_to_a_single_file() {
    let workspace = SampleWorkspace::multiple_hardlinks_to_a_single_file();

    let json = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--json-output")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData");

    let actual_size = match &json.unit_and_tree {
        UnitAndTree::Blocks(_) => panic!("expecting Bytes, but got {:?}", &json.unit_and_tree),
        UnitAndTree::Bytes(tree) => tree.size,
    };

    let expected_size = workspace
        .join("file.txt")
        .pipe_as_ref(read_apparent_size)
        .mul(11)
        .add(read_apparent_size(&workspace))
        .pipe(Bytes::new);

    assert_eq!(actual_size, expected_size);
}
