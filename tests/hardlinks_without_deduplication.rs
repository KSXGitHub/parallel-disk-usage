#![cfg(unix)] // This feature is not available in Windows
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use parallel_disk_usage::{
    json_data::{JsonData, JsonTree},
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
fn multiple_hardlinks_to_a_single_file() {
    let links = 10;
    let workspace = SampleWorkspace::multiple_hardlinks_to_a_single_file(100_000, links);

    let tree = Command::new(PDU)
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
        .expect("get tree of bytes");

    let actual_size = tree.size;
    let expected_size = workspace
        .join("file.txt")
        .pipe_as_ref(read_apparent_size)
        .mul(links + 1)
        .add(read_apparent_size(&workspace))
        .pipe(Bytes::new);
    assert_eq!(actual_size, expected_size);

    assert_eq!(tree.shared.details, None);
    assert_eq!(tree.shared.summary, None);
}

#[test]
fn complex_tree_with_shared_and_unique_files() {
    let files_per_branch = 2 * 3 * 4;
    let workspace =
        SampleWorkspace::complex_tree_with_shared_and_unique_files(files_per_branch, 100_000);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--min-ratio=0")
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
        .expect("get tree of bytes");

    let file_size = workspace
        .join("no-hardlinks/file-0.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

    let inode_size = |path: &str| {
        workspace
            .join(path)
            .pipe_as_ref(read_apparent_size)
            .pipe(Bytes::new)
    };

    let actual_size = tree.size;
    let expected_size: Bytes = [
        inode_size("."),
        inode_size("no-hardlinks"),
        inode_size("some-hardlinks"),
        inode_size("only-hardlinks"),
        inode_size("only-hardlinks/exclusive"),
        inode_size("only-hardlinks/mixed"),
        inode_size("only-hardlinks/external"),
        file_size * files_per_branch, // no-hardlinks/*
        file_size
            * [
                3 * files_per_branch / 8,
                2 * files_per_branch / 8,
                files_per_branch / 8,
                files_per_branch / 8,
                files_per_branch * 4 / 8,
            ]
            .into_iter()
            .sum::<usize>(), // some-hardlinks/*
        file_size * (2 * files_per_branch), // only-hardlinks/exclusive/*
        file_size * (files_per_branch / 2 + 2 * files_per_branch / 2), // only-hardlinks/mixed/*
        file_size * files_per_branch, // only-hardlinks/external/*
    ]
    .into_iter()
    .sum();
    assert_eq!(actual_size, expected_size);

    assert_eq!(tree.shared.details, None);
    assert_eq!(tree.shared.summary, None);
}

#[test]
fn hardlinks_and_non_hardlinks() {
    let files_per_branch = 2 * 4;
    let workspace =
        SampleWorkspace::complex_tree_with_shared_and_unique_files(files_per_branch, 100_000);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--min-ratio=0")
        .with_arg("--quantity=apparent-size")
        .with_arg("--json-output")
        .with_arg("some-hardlinks")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text)
        .pipe_as_ref(serde_json::from_str::<JsonData>)
        .expect("parse stdout as JsonData")
        .body
        .pipe(JsonTree::<Bytes>::try_from)
        .expect("get tree of bytes");

    let file_size = workspace
        .join("some-hardlinks/file-0.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

    let inode_size = |path: &str| {
        workspace
            .join(path)
            .pipe_as_ref(read_apparent_size)
            .pipe(Bytes::new)
    };

    let actual_size = tree.size;
    let expected_size = [
        inode_size("some-hardlinks"),
        file_size * files_per_branch,  // file-{index}.txt
        file_size * (2usize + 1usize), // link0-file0.txt, link1-file0.txt, link0-file1.txt
    ]
    .into_iter()
    .sum();
    assert_eq!(actual_size, expected_size);

    assert_eq!(tree.shared.details, None);
    assert_eq!(tree.shared.summary, None);

    let visualization = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("some-hardlinks")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("STDOUT:\n{visualization}");
    assert!(!visualization.contains("Hardlinks detected!"));
}
