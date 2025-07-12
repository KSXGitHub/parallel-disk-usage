#![cfg(unix)] // This feature is not available in Windows
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use parallel_disk_usage::{
    data_tree::Reflection,
    json_data::{JsonData, UnitAndTree},
    size::Bytes,
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    iter,
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

    let UnitAndTree::Bytes(tree) = &json.unit_and_tree else {
        panic!("expecting Bytes but got {:?}", &json.unit_and_tree);
    };

    let file_size = workspace
        .join("file.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

    let actual_size = tree.size;
    let expected_size = workspace
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new)
        .add(file_size);
    assert_eq!(actual_size, expected_size);

    let actual_children = {
        let mut children = tree.children.clone();
        children.sort_by(|a, b| a.name.cmp(&b.name));
        children
    };
    let expected_children: Vec<_> = {
        let links = (0..10).map(|num| format!("link.{num}"));
        let node = |name| Reflection {
            name,
            size: file_size,
            children: Vec::new(),
        };
        "file.txt"
            .to_string()
            .pipe(iter::once)
            .chain(links)
            .map(node)
            .collect()
    };
    assert_eq!(actual_children, expected_children);
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
