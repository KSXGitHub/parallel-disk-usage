#![cfg(unix)] // This feature is not available in Windows
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use parallel_disk_usage::{
    data_tree::Reflection,
    json_data::{JsonData, JsonTree},
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
fn multiple_hardlinks_to_a_single_file_with_deduplication() {
    let links = 10;
    let workspace = SampleWorkspace::multiple_hardlinks_to_a_single_file(100_000, links);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("--json-output")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .body
        .pipe(JsonTree::<Bytes>::try_from)
        .expect("get tree of bytes");

    let actual_size = tree.size;

    let file_size = workspace
        .join("file.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

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
        let links = (0..links).map(|num| format!("link.{num}"));
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
fn multiple_hardlinks_to_a_single_file_without_deduplication() {
    let links = 10;
    let workspace = SampleWorkspace::multiple_hardlinks_to_a_single_file(100_000, links);

    let actual_size = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--json-output")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .body
        .pipe(JsonTree::<Bytes>::try_from)
        .expect("get tree of bytes")
        .size;

    let expected_size = workspace
        .join("file.txt")
        .pipe_as_ref(read_apparent_size)
        .mul(links + 1)
        .add(read_apparent_size(&workspace))
        .pipe(Bytes::new);

    assert_eq!(actual_size, expected_size);
}
