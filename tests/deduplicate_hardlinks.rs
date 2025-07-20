#![cfg(unix)] // This feature is not available in Windows
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use parallel_disk_usage::{
    data_tree::Reflection,
    hardlink::{
        hardlink_list::{self, Summary},
        LinkPathListReflection,
    },
    inode::InodeNumber,
    json_data::{JsonData, JsonTree},
    size::Bytes,
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    collections::HashSet,
    iter,
    ops::{Add, Mul},
    path::PathBuf,
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

    let file_size = workspace
        .join("file.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

    let file_ino = workspace
        .join("file.txt")
        .pipe_as_ref(read_ino)
        .pipe(InodeNumber::from);

    let actual_size = tree.size;
    let expected_size = workspace
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new)
        .add(file_size);
    assert_eq!(actual_size, expected_size);

    let actual_children = tree
        .children
        .clone()
        .into_sorted_by(|a, b| a.name.cmp(&b.name));
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

    let actual_shared_details: Vec<_> = tree
        .shared
        .details
        .expect("get details")
        .iter()
        .cloned()
        .collect();
    let expected_shared_details = vec![hardlink_list::reflection::ReflectionEntry {
        ino: file_ino,
        size: file_size,
        links: 1 + links,
        paths: (0..links)
            .map(|num| format!("./link.{num}"))
            .chain("./file.txt".to_string().pipe(iter::once))
            .map(PathBuf::from)
            .collect::<HashSet<_>>()
            .pipe(LinkPathListReflection),
    }];
    assert_eq!(actual_shared_details, expected_shared_details);

    let actual_shared_summary = tree.shared.summary;
    let expected_shared_summary = {
        let mut summary = Summary::default();
        summary.inodes = 1;
        summary.exclusive_inodes = 1;
        summary.all_links = 1 + links;
        summary.detected_links = 1 + links as usize;
        summary.exclusive_links = 1 + links as usize;
        summary.shared_size = file_size;
        summary.exclusive_shared_size = file_size;
        Some(summary)
    };
    assert_eq!(actual_shared_summary, expected_shared_summary);
}

#[test]
fn multiple_hardlinks_to_a_single_file_without_deduplication() {
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

    assert!(tree.shared.details.is_none());
    assert_eq!(tree.shared.summary, None);
}

#[test]
fn complex_tree_with_shared_and_unique_files_with_deduplication() {
    let files_per_branch = 255;
    let workspace =
        SampleWorkspace::complex_tree_with_shared_and_unique_files(files_per_branch, 100_000);

    let tree = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--min-ratio=0")
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
        .join("no-hardlinks/file-0.txt")
        .pipe_as_ref(read_apparent_size)
        .pipe(Bytes::new);

    let inode_size = |path: &str| {
        workspace
            .join(path)
            .pipe_as_ref(read_apparent_size)
            .pipe(Bytes::new)
    };

    // The following formula treat the first file as "real" and
    // the non-first file with the same inode as "fake" for ease
    // of reasoning.
    // It should still produce the same result as the proper
    // deduplication formula however.
    #[expect(clippy::erasing_op)]
    let expected_size: Bytes = [
        inode_size("."),
        inode_size("no-hardlinks"),
        inode_size("some-hardlinks"),
        inode_size("only-hardlinks"),
        inode_size("only-hardlinks/exclusive"),
        inode_size("only-hardlinks/mixed"),
        inode_size("only-hardlinks/external"),
        file_size * files_per_branch, // no-hardlinks/*
        file_size * files_per_branch, // some-hardlinks/*
        file_size * files_per_branch, // only-hardlinks/exclusive/*
        file_size * files_per_branch, // only-hardlinks/mixed/*
        file_size * 0usize,           // only-hardlinks/external/*
    ]
    .into_iter()
    .sum();

    assert_eq!(actual_size, expected_size);
}
